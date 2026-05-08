using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Maze.Maui.App.Services;
using System.Collections.ObjectModel;
using System.Net;
using System.Text.RegularExpressions;
using System.Threading;

namespace Maze.Maui.App.ViewModels
{
    /// <summary>
    /// Backing ViewModel for the Email Addresses panel inside the Account
    /// surface. Owns the list of rows, the add-form state, and the
    /// commands the per-row actions invoke. Optimistic-update pattern
    /// matches the React panel: snapshot before mutating, revert on error.
    /// </summary>
    public partial class EmailAddressesViewModel : BaseViewModel
    {
        private readonly IAuthService _authService;

        // Same regex used by the React frontend's isValidEmail helper —
        // keeps client-side validation behaviour symmetric across UIs.
        private static readonly Regex EmailFormat = new(@"^[^@\s]+@[^@\s]+\.[^@\s]+$", RegexOptions.Compiled);

        public ObservableCollection<EmailRowViewModel> Emails { get; } = new();

        [ObservableProperty]
        [NotifyCanExecuteChangedFor(nameof(AddEmailCommand))]
        private string newEmail = "";

        [ObservableProperty]
        private string errorMessage = "";

        [ObservableProperty]
        private string loadStatus = "";

        /// <summary>Transient panel-level success message shown after a
        /// Resend Verification request succeeds. Auto-clears after
        /// <see cref="ResendFlashDuration"/> via a cancellable Task.Delay
        /// — a fresh Resend invocation cancels any in-flight clear so the
        /// new message isn't wiped by the previous timer firing.</summary>
        [ObservableProperty]
        private string resendFlash = "";

        // 5 seconds is the same fade window used by the React panel — long
        // enough to read, short enough that two consecutive Resends don't
        // pile up.
        internal static readonly TimeSpan ResendFlashDuration = TimeSpan.FromSeconds(5);
        private CancellationTokenSource? _resendFlashCts;

        public EmailAddressesViewModel(IAuthService authService)
        {
            Title = "Email Addresses";
            _authService = authService;
        }

        partial void OnNewEmailChanged(string value) => ErrorMessage = "";

        /// <summary>True when the typed address passes a basic format check
        /// and no other write is in flight. Drives the Add button's
        /// enabled state.</summary>
        public bool CanAddEmail => !IsBusy && EmailFormat.IsMatch(NewEmail);

        protected override void OnPropertyChanged(System.ComponentModel.PropertyChangedEventArgs e)
        {
            base.OnPropertyChanged(e);
            // IsBusy isn't a [NotifyCanExecuteChangedFor] target on Add (that
            // attribute is on NewEmail) — bridge it manually so the Add
            // button disables correctly during a write.
            if (e.PropertyName == nameof(IsBusy))
            {
                AddEmailCommand.NotifyCanExecuteChanged();
                OnPropertyChanged(nameof(CanAddEmail));
            }
            if (e.PropertyName == nameof(NewEmail))
            {
                OnPropertyChanged(nameof(CanAddEmail));
            }
        }

        [RelayCommand]
        private async Task LoadEmails()
        {
            if (IsBusy) return;
            IsBusy = true;
            ErrorMessage = "";
            LoadStatus = "Loading emails...";
            try
            {
                var rows = await _authService.GetMyEmailsAsync();
                ReplaceRows(rows);
                LoadStatus = "";
            }
            catch
            {
                ErrorMessage = "Failed to load emails. Please try again.";
                LoadStatus = "";
            }
            finally
            {
                IsBusy = false;
            }
        }

        private bool CanAddEmailExecute() => CanAddEmail;

        [RelayCommand(CanExecute = nameof(CanAddEmailExecute))]
        private async Task AddEmail()
        {
            // Non-optimistic: server-supplied verified/verified_at come back
            // in the response, so wait for the round-trip and reconcile.
            IsBusy = true;
            ErrorMessage = "";
            try
            {
                var rows = await _authService.AddEmailAsync(NewEmail);
                ReplaceRows(rows);
                NewEmail = "";
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Conflict)
            {
                ErrorMessage = "That email is already in use";
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.BadRequest)
            {
                ErrorMessage = "Email format is invalid";
            }
            catch
            {
                ErrorMessage = "Failed to add email. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        [RelayCommand]
        private async Task RemoveEmail(EmailRowViewModel? row)
        {
            if (row is null || IsBusy) return;
            // Optimistic snapshot: if the server rejects, restore exactly
            // what was on screen so the user doesn't see a phantom edit.
            var previous = Emails.ToList();
            IsBusy = true;
            ErrorMessage = "";
            Emails.Remove(row);
            try
            {
                var rows = await _authService.RemoveEmailAsync(row.Email);
                ReplaceRows(rows);
            }
            catch
            {
                RestoreRows(previous);
                ErrorMessage = "Failed to remove email. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        [RelayCommand]
        private async Task SetPrimary(EmailRowViewModel? row)
        {
            if (row is null || IsBusy) return;
            var previous = Emails.ToList();
            // Snapshot the previous primary/verified flags too so a revert
            // restores the full row state, not just collection identity.
            var previousPrimary = previous
                .Select(r => (r.Email, r.IsPrimary, r.Verified))
                .ToList();
            IsBusy = true;
            ErrorMessage = "";
            foreach (var existing in Emails)
            {
                existing.IsPrimary = existing.Email == row.Email;
            }
            try
            {
                var rows = await _authService.SetPrimaryEmailAsync(row.Email);
                ReplaceRows(rows);
            }
            catch (HttpRequestException ex) when (ex.StatusCode == HttpStatusCode.Conflict)
            {
                RestoreFromSnapshot(previousPrimary);
                ErrorMessage = "An unverified email cannot be promoted to primary";
            }
            catch
            {
                RestoreFromSnapshot(previousPrimary);
                ErrorMessage = "Failed to set primary email. Please try again.";
            }
            finally
            {
                IsBusy = false;
            }
        }

        [RelayCommand]
        private async Task VerifyEmail(EmailRowViewModel? row)
        {
            if (row is null || IsBusy) return;
            IsBusy = true;
            ErrorMessage = "";
            ResendFlash = "";
            // Cancel any prior auto-clear so a rapid second Resend doesn't
            // see its own message wiped by the first timer firing.
            _resendFlashCts?.Cancel();
            try
            {
                await _authService.RequestEmailVerificationAsync(row.Email);
                ResendFlash = $"Verification link sent to {row.Email}.";
                _resendFlashCts = new CancellationTokenSource();
                _ = ScheduleResendFlashClearAsync(row.Email, _resendFlashCts.Token);
            }
            catch
            {
                ErrorMessage = "Failed to resend verification link";
            }
            finally
            {
                IsBusy = false;
            }
        }

        private async Task ScheduleResendFlashClearAsync(string email, CancellationToken ct)
        {
            try
            {
                await Task.Delay(ResendFlashDuration, ct);
            }
            catch (OperationCanceledException)
            {
                return;
            }
            // Only clear if the message we set is still the one on screen —
            // a subsequent Resend may have replaced it with a different
            // address before the timer fired.
            if (ResendFlash == $"Verification link sent to {email}.")
            {
                ResendFlash = "";
            }
        }

        private void ReplaceRows(List<UserEmail> rows)
        {
            Emails.Clear();
            foreach (var r in rows)
            {
                Emails.Add(new EmailRowViewModel
                {
                    Email = r.Email,
                    IsPrimary = r.IsPrimary,
                    Verified = r.Verified,
                });
            }
        }

        private void RestoreRows(List<EmailRowViewModel> previous)
        {
            Emails.Clear();
            foreach (var r in previous) Emails.Add(r);
        }

        private void RestoreFromSnapshot(List<(string Email, bool IsPrimary, bool Verified)> snapshot)
        {
            Emails.Clear();
            foreach (var (email, isPrimary, verified) in snapshot)
            {
                Emails.Add(new EmailRowViewModel
                {
                    Email = email,
                    IsPrimary = isPrimary,
                    Verified = verified,
                });
            }
        }
    }
}
