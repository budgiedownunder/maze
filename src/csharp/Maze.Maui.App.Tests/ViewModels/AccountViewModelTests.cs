using System.Net;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the account profile load/save lifecycle, the welcome-mode
    /// flag, the dirty-tracking <c>CanSaveProfile</c> guard, the password
    /// optimistic-update message handler, and the delete-account
    /// confirmation flow.
    /// </summary>
    public class AccountViewModelTests
    {
        private static (AccountViewModel vm, Mock<IAuthService> auth, Mock<IDialogService> dialog, Mock<INavigationService> nav)
            BuildVm()
        {
            var auth = new Mock<IAuthService>();
            var dialog = new Mock<IDialogService>();
            var nav = new Mock<INavigationService>();
            var avatar = new Mock<IAvatarService>();
            var picker = new Mock<IImagePickerService>();
            var vm = new AccountViewModel(auth.Object, dialog.Object, nav.Object, avatar.Object, picker.Object);
            return (vm, auth, dialog, nav);
        }

        private static UserProfile MakeProfile(string username = "alice", string fullName = "Alice Example", bool isAdmin = false, bool hasPassword = true) =>
            new() { Username = username, FullName = fullName, IsAdmin = isAdmin, HasPassword = hasPassword };

        // ---- LoadProfile ----------------------------------------------------

        [Fact]
        public async Task LoadProfile_PopulatesFieldsFromServer()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync())
                .ReturnsAsync(MakeProfile("alice", "Alice Example", isAdmin: true, hasPassword: false));

            await vm.LoadProfileCommand.ExecuteAsync(null);

            Assert.Equal("alice", vm.Username);
            Assert.Equal("Alice Example", vm.FullName);
            Assert.True(vm.IsAdmin);
            Assert.False(vm.HasPassword);
            Assert.Equal("", vm.LoadStatus);
            Assert.Equal("", vm.ErrorMessage);
        }

        [Fact]
        public async Task LoadProfile_OnFailure_SetsErrorMessage()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync()).ThrowsAsync(new HttpRequestException("boom"));

            await vm.LoadProfileCommand.ExecuteAsync(null);

            Assert.Contains("Failed to load profile", vm.ErrorMessage);
        }

        [Fact]
        public async Task LoadProfile_SetsAvatarBytesFromAvatarService()
        {
            var auth = new Mock<IAuthService>();
            var avatar = new Mock<IAvatarService>();
            byte[] bytes = { 1, 2, 3 };
            auth.Setup(a => a.GetMyProfileAsync())
                .ReturnsAsync(new UserProfile { Id = "u1", Username = "alice", AvatarUpdatedAt = "2026-06-16T12:00:00Z" });
            avatar.Setup(s => s.TryLoadAvatarBytesAsync("u1", "2026-06-16T12:00:00Z")).ReturnsAsync(bytes);
            var vm = new AccountViewModel(auth.Object, new Mock<IDialogService>().Object, new Mock<INavigationService>().Object, avatar.Object, new Mock<IImagePickerService>().Object);

            await vm.LoadProfileCommand.ExecuteAsync(null);

            Assert.Same(bytes, vm.AvatarBytes);
        }

        [Fact]
        public async Task LoadProfile_NoAvatar_LeavesAvatarBytesNull()
        {
            var auth = new Mock<IAuthService>();
            var avatar = new Mock<IAvatarService>();
            auth.Setup(a => a.GetMyProfileAsync())
                .ReturnsAsync(new UserProfile { Id = "u1", Username = "alice", AvatarUpdatedAt = null });
            // No marker → the service resolves to null (no fetch); the VM stores it.
            avatar.Setup(s => s.TryLoadAvatarBytesAsync(It.IsAny<string>(), null)).ReturnsAsync((byte[]?)null);
            var vm = new AccountViewModel(auth.Object, new Mock<IDialogService>().Object, new Mock<INavigationService>().Object, avatar.Object, new Mock<IImagePickerService>().Object);

            await vm.LoadProfileCommand.ExecuteAsync(null);

            Assert.Null(vm.AvatarBytes);
        }

        // ---- ChangeAvatar / RemoveAvatar ------------------------------------

        // Builds an account VM whose profile loads as user "u1" with no avatar,
        // returning the avatar + picker mocks for the test to drive.
        private static (AccountViewModel vm, Mock<IAvatarService> avatar, Mock<IImagePickerService> picker)
            BuildAvatarVm()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(new UserProfile { Id = "u1", Username = "alice" });
            var avatar = new Mock<IAvatarService>();
            // No marker on load → null bytes (avoids Moq's empty-array default).
            avatar.Setup(s => s.TryLoadAvatarBytesAsync(It.IsAny<string>(), null)).ReturnsAsync((byte[]?)null);
            var picker = new Mock<IImagePickerService>();
            var vm = new AccountViewModel(auth.Object, new Mock<IDialogService>().Object, new Mock<INavigationService>().Object, avatar.Object, picker.Object);
            return (vm, avatar, picker);
        }

        [Fact]
        public async Task ChangeAvatar_UploadsPickedImageAndSetsBytes()
        {
            var (vm, avatar, picker) = BuildAvatarVm();
            await vm.LoadProfileCommand.ExecuteAsync(null); // sets _userId = "u1"
            picker.Setup(p => p.PickImageAsync()).ReturnsAsync(new PickedImage(new byte[] { 1, 2, 3 }, "image/png"));
            avatar.Setup(s => s.UploadAvatarAsync(It.IsAny<byte[]>(), "image/png")).ReturnsAsync("2026-06-16T12:00:00Z");
            byte[] canonical = { 9, 9, 9 };
            avatar.Setup(s => s.TryLoadAvatarBytesAsync("u1", "2026-06-16T12:00:00Z")).ReturnsAsync(canonical);

            await vm.ChangeAvatarCommand.ExecuteAsync(null);

            Assert.Same(canonical, vm.AvatarBytes);
            Assert.True(vm.HasAvatar);
            Assert.Equal("", vm.AvatarError);
        }

        [Fact]
        public async Task ChangeAvatar_RejectsNonImage_WithoutUploading()
        {
            var (vm, avatar, picker) = BuildAvatarVm();
            await vm.LoadProfileCommand.ExecuteAsync(null);
            picker.Setup(p => p.PickImageAsync()).ReturnsAsync(new PickedImage(new byte[] { 1 }, "image/gif"));

            await vm.ChangeAvatarCommand.ExecuteAsync(null);

            Assert.Contains("PNG or JPEG", vm.AvatarError);
            avatar.Verify(s => s.UploadAvatarAsync(It.IsAny<byte[]>(), It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task ChangeAvatar_RejectsOversize_WithoutUploading()
        {
            var (vm, avatar, picker) = BuildAvatarVm();
            await vm.LoadProfileCommand.ExecuteAsync(null);
            picker.Setup(p => p.PickImageAsync()).ReturnsAsync(new PickedImage(new byte[2 * 1024 * 1024 + 1], "image/png"));

            await vm.ChangeAvatarCommand.ExecuteAsync(null);

            Assert.Contains("2 MB", vm.AvatarError);
            avatar.Verify(s => s.UploadAvatarAsync(It.IsAny<byte[]>(), It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task ChangeAvatar_WhenCancelled_DoesNothing()
        {
            var (vm, avatar, picker) = BuildAvatarVm();
            picker.Setup(p => p.PickImageAsync()).ReturnsAsync((PickedImage?)null);

            await vm.ChangeAvatarCommand.ExecuteAsync(null);

            Assert.Equal("", vm.AvatarError);
            avatar.Verify(s => s.UploadAvatarAsync(It.IsAny<byte[]>(), It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task RemoveAvatar_OnSuccess_ClearsBytes()
        {
            var (vm, avatar, picker) = BuildAvatarVm();
            await vm.LoadProfileCommand.ExecuteAsync(null);
            // Upload one first so there's something to remove.
            picker.Setup(p => p.PickImageAsync()).ReturnsAsync(new PickedImage(new byte[] { 1 }, "image/png"));
            avatar.Setup(s => s.UploadAvatarAsync(It.IsAny<byte[]>(), It.IsAny<string>())).ReturnsAsync("m1");
            avatar.Setup(s => s.TryLoadAvatarBytesAsync("u1", "m1")).ReturnsAsync(new byte[] { 7 });
            await vm.ChangeAvatarCommand.ExecuteAsync(null);
            Assert.True(vm.HasAvatar);

            avatar.Setup(s => s.DeleteAvatarAsync()).ReturnsAsync(true);
            await vm.RemoveAvatarCommand.ExecuteAsync(null);

            Assert.Null(vm.AvatarBytes);
            Assert.False(vm.HasAvatar);
        }

        // ---- SaveProfile ----------------------------------------------------

        [Fact]
        public async Task SaveProfile_CallsUpdateProfileAndRebasesBaseline()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(MakeProfile("alice", "Alice"));
            await vm.LoadProfileCommand.ExecuteAsync(null);

            auth.Setup(a => a.UpdateProfileAsync("alice2", "Alice2"))
                .ReturnsAsync(MakeProfile("alice2", "Alice2"));
            vm.Username = "alice2";
            vm.FullName = "Alice2";

            await vm.SaveProfileCommand.ExecuteAsync(null);

            auth.Verify(a => a.UpdateProfileAsync("alice2", "Alice2"), Times.Once);
            // After save, baseline has rebased — a no-op resave should not be allowed.
            Assert.False(vm.SaveProfileCommand.CanExecute(null));
        }

        [Fact]
        public async Task SaveProfile_409Conflict_ShowsUsernameTakenMessage()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(MakeProfile("alice", "Alice"));
            await vm.LoadProfileCommand.ExecuteAsync(null);

            auth.Setup(a => a.UpdateProfileAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("conflict", null, HttpStatusCode.Conflict));
            vm.Username = "taken";

            await vm.SaveProfileCommand.ExecuteAsync(null);

            Assert.Contains("already in use", vm.ErrorMessage);
        }

        [Fact]
        public async Task SaveProfile_GenericFailure_ShowsGenericErrorMessage()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(MakeProfile("alice", "Alice"));
            await vm.LoadProfileCommand.ExecuteAsync(null);

            auth.Setup(a => a.UpdateProfileAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new InvalidOperationException("network"));
            vm.Username = "alice2";

            await vm.SaveProfileCommand.ExecuteAsync(null);

            Assert.Contains("Failed to save profile", vm.ErrorMessage);
        }

        // ---- CanSaveProfile -------------------------------------------------

        [Fact]
        public void CanSaveProfile_FalseWhenUsernameEmpty()
        {
            var (vm, _, _, _) = BuildVm();
            vm.Username = "";
            vm.FullName = "Whatever";
            Assert.False(vm.SaveProfileCommand.CanExecute(null));
        }

        [Fact]
        public async Task CanSaveProfile_FalseWhenNoChangeFromBaseline()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(MakeProfile("alice", "Alice"));
            await vm.LoadProfileCommand.ExecuteAsync(null);
            // Setting back to the same baseline values must not enable Save.
            vm.Username = "alice";
            vm.FullName = "Alice";
            Assert.False(vm.SaveProfileCommand.CanExecute(null));
        }

        [Fact]
        public async Task CanSaveProfile_TrueAfterChangeFromBaseline()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(MakeProfile("alice", "Alice"));
            await vm.LoadProfileCommand.ExecuteAsync(null);
            vm.FullName = "Alice Renamed";
            Assert.True(vm.SaveProfileCommand.CanExecute(null));
        }

        // ---- ChangePassword navigation -------------------------------------

        [Fact]
        public async Task ChangePassword_NavigatesWithCurrentHasPasswordFlag()
        {
            var (vm, _, _, nav) = BuildVm();
            vm.HasPassword = false;

            await vm.ChangePasswordCommand.ExecuteAsync(null);

            nav.Verify(n => n.GoToAsync(
                nameof(ChangePasswordPage),
                It.Is<IDictionary<string, object>>(d => (bool)d["HasPassword"] == false)),
                Times.Once);
        }

        // ---- DeleteAccount --------------------------------------------------

        [Fact]
        public async Task DeleteAccount_UserCancels_DoesNothing()
        {
            var (vm, auth, dialog, nav) = BuildVm();
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(false);

            await vm.DeleteAccountCommand.ExecuteAsync(null);

            auth.Verify(a => a.DeleteMyAccountAsync(), Times.Never);
            nav.Verify(n => n.GoToRootAsync(It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task DeleteAccount_UserConfirms_DeletesAndPublishesMazesInvalidated()
        {
            var (vm, auth, dialog, nav) = BuildVm();
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(true);
            int received = 0;
            object recipient = new();
            WeakReferenceMessenger.Default.Register<MazesInvalidatedMessage>(recipient, (_, _) => received++);
            try
            {
                await vm.DeleteAccountCommand.ExecuteAsync(null);
            }
            finally
            {
                WeakReferenceMessenger.Default.Unregister<MazesInvalidatedMessage>(recipient);
            }

            auth.Verify(a => a.DeleteMyAccountAsync(), Times.Once);
            nav.Verify(n => n.GoToRootAsync("//LoginPage"), Times.Once);
            Assert.Equal(1, received);
        }

        [Fact]
        public async Task DeleteAccount_ServerFailure_ShowsErrorAndDoesNotNavigate()
        {
            var (vm, auth, dialog, nav) = BuildVm();
            dialog.Setup(d => d.ShowConfirmation(It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                  .ReturnsAsync(true);
            auth.Setup(a => a.DeleteMyAccountAsync()).ThrowsAsync(new HttpRequestException("boom"));

            await vm.DeleteAccountCommand.ExecuteAsync(null);

            Assert.Contains("Failed to delete account", vm.ErrorMessage);
            nav.Verify(n => n.GoToRootAsync(It.IsAny<string>()), Times.Never);
        }

        // ---- ClearProfile ---------------------------------------------------

        [Fact]
        public async Task ClearProfile_ResetsAllFieldsAndSetsLoadingStatus()
        {
            var (vm, auth, _, _) = BuildVm();
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(MakeProfile("alice", "Alice", isAdmin: true));
            await vm.LoadProfileCommand.ExecuteAsync(null);

            vm.ClearProfile();

            Assert.Equal("", vm.Username);
            Assert.Equal("", vm.FullName);
            Assert.False(vm.IsAdmin);
            Assert.Equal("", vm.ErrorMessage);
            Assert.Equal("Loading profile...", vm.LoadStatus);
        }

        // ---- PasswordSetMessage handler ------------------------------------

        [Fact]
        public void Receive_PasswordSetMessage_FlipsHasPasswordTrue()
        {
            var (vm, _, _, _) = BuildVm();
            vm.HasPassword = false;

            WeakReferenceMessenger.Default.Send(new PasswordSetMessage());

            Assert.True(vm.HasPassword);
        }
    }
}
