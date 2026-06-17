using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Services;
using Maze.Maui.App.Views;
using System.Net;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Represents the view model for the account page
    /// </summary>
    public partial class AccountViewModel : BaseViewModel,
        IRecipient<PasswordSetMessage>
    {
        private const int MaxAvatarBytes = 2 * 1024 * 1024;

        private readonly IAuthService _authService;
        private readonly IDialogService _dialogService;
        private readonly INavigationService _navigationService;
        private readonly IAvatarService _avatarService;
        private readonly IImagePickerService _imagePickerService;

        private string _loadedUsername = "";
        private string _loadedFullName = "";
        private string _userId = "";

        /// <summary>
        /// The signed-in user's avatar PNG bytes, or <c>null</c> when they have
        /// none (the avatar control then shows the generic placeholder). Bound by
        /// the Shell flyout header and the account page so both reflect the
        /// current user. Bytes (not a UI image type) so this view model stays
        /// free of MAUI runtime types.
        /// </summary>
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(HasAvatar))]
        private byte[]? avatarBytes;

        /// <summary>Whether the user currently has an avatar (drives the Remove button).</summary>
        public bool HasAvatar => AvatarBytes is not null;

        /// <summary>Whether an avatar upload/remove is in flight (disables the avatar buttons).</summary>
        [ObservableProperty]
        private bool avatarBusy;

        /// <summary>Inline error for the avatar upload/remove flow.</summary>
        [ObservableProperty]
        private string avatarError = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SaveProfileCommand))]
        private string username = "";

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(SaveProfileCommand))]
        private string fullName = "";

        [ObservableProperty]
        private bool isAdmin;

        /// <summary>
        /// Whether the authenticated user has a password set. Drives the
        /// trigger-button label ("Change Password" vs "Set Password") and
        /// the variant the popup renders. Flipped optimistically to
        /// <c>true</c> on receipt of <see cref="PasswordSetMessage"/> so
        /// the label updates without re-fetching the profile.
        /// </summary>
        [ObservableProperty]
        private bool hasPassword = true;

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private string loadStatus = "";

        /// <summary>
        /// When true, the AccountPage renders a one-line welcome banner above
        /// the form. Set by the OAuth sign-in flow when the server signals
        /// <c>new_user=true</c>; cleared by <c>AccountPage.OnDisappearing</c>
        /// so subsequent burger-menu opens of the Account page don't keep
        /// showing the banner.
        /// </summary>
        [ObservableProperty]
        private bool isWelcomeMode;

        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="authService">Injected auth service</param>
        /// <param name="dialogService">Injected dialog service</param>
        /// <param name="navigationService">Injected navigation service</param>
        /// <param name="avatarService">Injected avatar service</param>
        /// <param name="imagePickerService">Injected image-picker service</param>
        public AccountViewModel(IAuthService authService, IDialogService dialogService, INavigationService navigationService, IAvatarService avatarService, IImagePickerService imagePickerService)
        {
            Title = "Account";
            _authService = authService;
            _dialogService = dialogService;
            _navigationService = navigationService;
            _avatarService = avatarService;
            _imagePickerService = imagePickerService;
            // Subscribe to in-process pub/sub so a successful Set/Change in
            // the password popup flips the local HasPassword without a
            // re-fetch. Singleton lifetime guarantees we outlive any sender.
            WeakReferenceMessenger.Default.RegisterAll(this);
        }

        /// <inheritdoc/>
        public void Receive(PasswordSetMessage message) => HasPassword = true;

        /// <summary>
        /// Loads the user's profile from the server
        /// </summary>
        [RelayCommand]
        private async Task LoadProfile()
        {
            if (IsBusy)
                return;

            ClearProfile();
            IsBusy = true;
            try
            {
                var profile = await _authService.GetMyProfileAsync();
                _userId = profile.Id;
                Username = _loadedUsername = profile.Username;
                FullName = _loadedFullName = profile.FullName;
                IsAdmin = profile.IsAdmin;
                HasPassword = profile.HasPassword;
                AvatarBytes = await _avatarService.TryLoadAvatarBytesAsync(profile.Id, profile.AvatarUpdatedAt);
                LoadStatus = "";
            }
            catch
            {
                ErrorMessage = "Failed to load profile. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        /// <summary>
        /// Clears all profile fields and sets the load status message
        /// </summary>
        public void ClearProfile()
        {
            Username = _loadedUsername = "";
            FullName = _loadedFullName = "";
            IsAdmin = false;
            AvatarBytes = null;
            ErrorMessage = "";
            LoadStatus = "Loading profile...";
        }

        /// <summary>
        /// Picks an image, validates it (PNG/JPEG, &lt;= 2 MB) client-side,
        /// uploads it, and reloads the avatar so the account page and the Shell
        /// flyout header (same singleton view model) both update.
        /// </summary>
        [RelayCommand]
        private async Task ChangeAvatar()
        {
            if (AvatarBusy)
                return;

            AvatarError = "";
            PickedImage? picked = await _imagePickerService.PickImageAsync();
            if (picked is null)
                return; // user cancelled

            if (picked.ContentType != "image/png" && picked.ContentType != "image/jpeg")
            {
                AvatarError = "Please choose a PNG or JPEG image.";
                return;
            }
            if (picked.Bytes.Length > MaxAvatarBytes)
            {
                AvatarError = "Image must be 2 MB or smaller.";
                return;
            }

            AvatarBusy = true;
            try
            {
                string? marker = await _avatarService.UploadAvatarAsync(picked.Bytes, picked.ContentType);
                if (marker is null)
                {
                    AvatarError = "Failed to upload avatar. Please try again.";
                    return;
                }
                // Re-fetch the canonical (256x256 PNG) avatar the server produced.
                AvatarBytes = await _avatarService.TryLoadAvatarBytesAsync(_userId, marker);
            }
            finally
            {
                AvatarBusy = false;
            }
        }

        /// <summary>
        /// Removes the current avatar, updating the account page and flyout header.
        /// </summary>
        [RelayCommand]
        private async Task RemoveAvatar()
        {
            if (AvatarBusy)
                return;

            AvatarError = "";
            AvatarBusy = true;
            try
            {
                if (await _avatarService.DeleteAvatarAsync())
                    AvatarBytes = null;
                else
                    AvatarError = "Failed to remove avatar. Please try again.";
            }
            finally
            {
                AvatarBusy = false;
            }
        }

        partial void OnUsernameChanged(string value) => ErrorMessage = "";
        partial void OnFullNameChanged(string value) => ErrorMessage = "";

        private bool CanSaveProfile() =>
            !IsBusy &&
            !string.IsNullOrWhiteSpace(Username) &&
            (Username != _loadedUsername || FullName != _loadedFullName);

        /// <summary>
        /// Saves the user's updated profile to the server
        /// </summary>
        [RelayCommand(CanExecute = nameof(CanSaveProfile))]
        private async Task SaveProfile()
        {
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                var profile = await _authService.UpdateProfileAsync(Username, FullName);
                Username = _loadedUsername = profile.Username;
                FullName = _loadedFullName = profile.FullName;
                IsAdmin = profile.IsAdmin;
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Conflict)
            {
                ErrorMessage = "Username is already in use by another account";
            }
            catch
            {
                ErrorMessage = "Failed to save profile. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        /// <summary>
        /// Navigates to the change password page, passing the current
        /// <see cref="HasPassword"/> so the page renders the correct
        /// (Change vs Set) variant without re-fetching the profile.
        /// </summary>
        [RelayCommand]
        private async Task ChangePassword()
        {
            await _navigationService.GoToAsync(nameof(ChangePasswordPage), new Dictionary<string, object>
            {
                { "HasPassword", HasPassword }
            });
        }

        /// <summary>
        /// Confirms and deletes the user's account, then navigates to the login page
        /// </summary>
        [RelayCommand]
        private async Task DeleteAccount()
        {
            bool confirmed = await _dialogService.ShowConfirmation(
                "Delete Account",
                "Are you sure you want to permanently delete your account? This will also delete all your mazes and cannot be undone.",
                "Delete",
                "Cancel",
                isDestructive: true);

            if (!confirmed)
                return;

            IsBusy = true;
            ErrorMessage = "";
            try
            {
                await _authService.DeleteMyAccountAsync();
                WeakReferenceMessenger.Default.Send(new MazesInvalidatedMessage());
                ClearProfile();
                await _navigationService.GoToRootAsync("//LoginPage");
            }
            catch
            {
                ErrorMessage = "Failed to delete account. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }
    }
}
