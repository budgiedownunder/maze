using System.Net;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    public class EmailAddressesViewModelTests
    {
        private static readonly UserEmail PrimaryRow = new()
        {
            Email = "primary@example.com",
            IsPrimary = true,
            Verified = true,
            VerifiedAt = DateTimeOffset.UtcNow,
        };

        private static readonly UserEmail SecondaryRow = new()
        {
            Email = "second@example.com",
            IsPrimary = false,
            Verified = true,
            VerifiedAt = DateTimeOffset.UtcNow,
        };

        private static UserEmail NewVerifiedRow(string email) => new()
        {
            Email = email,
            IsPrimary = false,
            Verified = true,
            VerifiedAt = DateTimeOffset.UtcNow,
        };

        // ---- LoadEmails ----------------------------------------------------

        [Fact]
        public async Task LoadEmails_PopulatesRowsFromService()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.GetMyEmailsAsync())
                .ReturnsAsync(new List<UserEmail> { PrimaryRow, SecondaryRow });
            var vm = new EmailAddressesViewModel(auth.Object, NewDialog().Object);

            await vm.LoadEmailsCommand.ExecuteAsync(null);

            Assert.Equal(2, vm.Emails.Count);
            Assert.Equal("primary@example.com", vm.Emails[0].Email);
            Assert.True(vm.Emails[0].IsPrimary);
            Assert.False(vm.Emails[1].IsPrimary);
            Assert.Equal("", vm.ErrorMessage);
        }

        [Fact]
        public async Task LoadEmails_SetsErrorMessage_OnFailure()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.GetMyEmailsAsync())
                .ThrowsAsync(new HttpRequestException("boom"));
            var vm = new EmailAddressesViewModel(auth.Object, NewDialog().Object);

            await vm.LoadEmailsCommand.ExecuteAsync(null);

            Assert.Empty(vm.Emails);
            Assert.Contains("Failed to load", vm.ErrorMessage);
        }

        // ---- AddEmail ------------------------------------------------------

        [Fact]
        public void CanAddEmail_FalseUntilFormatValid()
        {
            var auth = new Mock<IAuthService>();
            var vm = new EmailAddressesViewModel(auth.Object, NewDialog().Object);

            Assert.False(vm.CanAddEmail);
            vm.NewEmail = "not-an-email";
            Assert.False(vm.CanAddEmail);
            vm.NewEmail = "good@example.com";
            Assert.True(vm.CanAddEmail);
        }

        [Fact]
        public async Task AddEmail_AppendsRowAndClearsInputOnSuccess()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.AddEmailAsync("second@example.com"))
                .ReturnsAsync(new List<UserEmail> { PrimaryRow, NewVerifiedRow("second@example.com") });
            var vm = new EmailAddressesViewModel(auth.Object, NewDialog().Object);
            vm.Emails.Add(new EmailRowViewModel { Email = PrimaryRow.Email, IsPrimary = true, Verified = true });
            vm.NewEmail = "second@example.com";

            await vm.AddEmailCommand.ExecuteAsync(null);

            Assert.Equal(2, vm.Emails.Count);
            Assert.Contains(vm.Emails, r => r.Email == "second@example.com");
            Assert.Equal("", vm.NewEmail);
        }

        [Fact]
        public async Task AddEmail_Surfaces409AsAlreadyInUseMessage()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.AddEmailAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("conflict", null, HttpStatusCode.Conflict));
            var vm = new EmailAddressesViewModel(auth.Object, NewDialog().Object)
            {
                NewEmail = "dup@example.com"
            };

            await vm.AddEmailCommand.ExecuteAsync(null);

            Assert.Contains("already in use", vm.ErrorMessage);
            // Input retained on error so the user can correct it.
            Assert.Equal("dup@example.com", vm.NewEmail);
        }

        [Fact]
        public async Task AddEmail_Surfaces400AsInvalidFormatMessage()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.AddEmailAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("bad", null, HttpStatusCode.BadRequest));
            var vm = new EmailAddressesViewModel(auth.Object, NewDialog().Object)
            {
                NewEmail = "weird@example.com"
            };

            await vm.AddEmailCommand.ExecuteAsync(null);

            Assert.Contains("Email format is invalid", vm.ErrorMessage);
        }

        // ---- RemoveEmail ---------------------------------------------------

        [Fact]
        public async Task RemoveEmail_PromptsForConfirmationAndRemovesRowWhenAccepted()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RemoveEmailAsync("second@example.com"))
                .ReturnsAsync(new List<UserEmail> { PrimaryRow });
            var dialog = NewDialog(confirm: true);
            var vm = SeedTwoRowVm(auth, dialog);
            var secondRow = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.RemoveEmailCommand.ExecuteAsync(secondRow);

            dialog.Verify(d => d.ShowConfirmation(
                "Remove email address",
                "Are you sure you want to remove 'second@example.com' from your account?",
                "Remove", "Cancel", true), Times.Once);
            Assert.Single(vm.Emails);
            Assert.Equal("primary@example.com", vm.Emails[0].Email);
        }

        [Fact]
        public async Task RemoveEmail_DoesNothingWhenUserCancelsConfirmation()
        {
            var auth = new Mock<IAuthService>();
            var dialog = NewDialog(confirm: false);
            var vm = SeedTwoRowVm(auth, dialog);
            var secondRow = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.RemoveEmailCommand.ExecuteAsync(secondRow);

            // List unchanged, no API call, no error surfaced.
            Assert.Equal(2, vm.Emails.Count);
            Assert.Contains(vm.Emails, r => r.Email == "second@example.com");
            auth.Verify(s => s.RemoveEmailAsync(It.IsAny<string>()), Times.Never);
            Assert.Equal("", vm.ErrorMessage);
        }

        [Fact]
        public async Task RemoveEmail_RestoresSnapshotOnFailure()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RemoveEmailAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("nope", null, HttpStatusCode.Conflict));
            var vm = SeedTwoRowVm(auth);
            var secondRow = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.RemoveEmailCommand.ExecuteAsync(secondRow);

            Assert.Equal(2, vm.Emails.Count);
            Assert.Contains(vm.Emails, r => r.Email == "second@example.com");
            Assert.Contains("Failed to remove", vm.ErrorMessage);
        }

        // ---- SetPrimary ----------------------------------------------------

        [Fact]
        public async Task SetPrimary_FlipsPrimaryFlagOnSuccess()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.SetPrimaryEmailAsync("second@example.com"))
                .ReturnsAsync(new List<UserEmail>
                {
                    new() { Email = "primary@example.com", IsPrimary = false, Verified = true },
                    new() { Email = "second@example.com", IsPrimary = true, Verified = true },
                });
            var vm = SeedTwoRowVm(auth);
            var secondRow = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.SetPrimaryCommand.ExecuteAsync(secondRow);

            Assert.True(vm.Emails.First(r => r.Email == "second@example.com").IsPrimary);
            Assert.False(vm.Emails.First(r => r.Email == "primary@example.com").IsPrimary);
        }

        [Fact]
        public async Task SetPrimary_RestoresPreviousPrimaryOnConflict()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.SetPrimaryEmailAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("unverified", null, HttpStatusCode.Conflict));
            var vm = SeedTwoRowVm(auth);
            var secondRow = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.SetPrimaryCommand.ExecuteAsync(secondRow);

            // Original primary is restored even though the optimistic update flipped it.
            Assert.True(vm.Emails.First(r => r.Email == "primary@example.com").IsPrimary);
            Assert.False(vm.Emails.First(r => r.Email == "second@example.com").IsPrimary);
            Assert.Contains("unverified", vm.ErrorMessage);
        }

        // ---- VerifyEmail ---------------------------------------------------

        [Fact]
        public async Task VerifyEmail_CallsRequestEmailVerificationOnceWithEmail()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RequestEmailVerificationAsync(It.IsAny<string>())).Returns(Task.CompletedTask);
            var vm = SeedTwoRowVm(auth);
            var row = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.VerifyEmailCommand.ExecuteAsync(row);

            auth.Verify(s => s.RequestEmailVerificationAsync("second@example.com"), Times.Once);
        }

        [Fact]
        public async Task VerifyEmail_SuccessSetsResendFlashWithEmail()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RequestEmailVerificationAsync(It.IsAny<string>())).Returns(Task.CompletedTask);
            var vm = SeedTwoRowVm(auth);
            var row = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.VerifyEmailCommand.ExecuteAsync(row);

            Assert.Contains("second@example.com", vm.ResendFlash);
            Assert.Equal("", vm.ErrorMessage);
        }

        [Fact]
        public async Task VerifyEmail_SuccessClearsPriorErrorMessage()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RequestEmailVerificationAsync(It.IsAny<string>())).Returns(Task.CompletedTask);
            var vm = SeedTwoRowVm(auth);
            vm.ErrorMessage = "stale error from a previous action";
            var row = vm.Emails.First();

            await vm.VerifyEmailCommand.ExecuteAsync(row);

            Assert.Equal("", vm.ErrorMessage);
        }

        [Fact]
        public async Task VerifyEmail_FailureSetsErrorAndClearsResendFlash()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RequestEmailVerificationAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("boom"));
            var vm = SeedTwoRowVm(auth);
            vm.ResendFlash = "stale flash from a previous resend";
            var row = vm.Emails.First();

            await vm.VerifyEmailCommand.ExecuteAsync(row);

            Assert.Contains("Failed to resend", vm.ErrorMessage);
            Assert.Equal("", vm.ResendFlash);
        }

        [Fact]
        public async Task VerifyEmail_RapidReResendReplacesFlashWithSecondAddress()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RequestEmailVerificationAsync(It.IsAny<string>())).Returns(Task.CompletedTask);
            var vm = SeedTwoRowVm(auth);
            var firstRow = vm.Emails.First(r => r.Email == "primary@example.com");
            var secondRow = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.VerifyEmailCommand.ExecuteAsync(firstRow);
            // Sanity: first invocation set the flash for the primary row.
            Assert.Contains("primary@example.com", vm.ResendFlash);

            await vm.VerifyEmailCommand.ExecuteAsync(secondRow);

            // Second invocation cancels the first auto-clear and replaces
            // the flash with the second address. The first timer firing 5s
            // later will see the message no longer matches its email and
            // no-op (covered by ScheduleResendFlashClearAsync's identity
            // check).
            Assert.Contains("second@example.com", vm.ResendFlash);
            Assert.DoesNotContain("primary@example.com", vm.ResendFlash);
        }

        // ---- helpers -------------------------------------------------------

        private static EmailAddressesViewModel SeedTwoRowVm(Mock<IAuthService> auth, Mock<IDialogService>? dialog = null)
        {
            var vm = new EmailAddressesViewModel(auth.Object, (dialog ?? NewDialog()).Object);
            vm.Emails.Add(new EmailRowViewModel { Email = "primary@example.com", IsPrimary = true, Verified = true });
            vm.Emails.Add(new EmailRowViewModel { Email = "second@example.com", IsPrimary = false, Verified = true });
            return vm;
        }

        // Default dialog mock confirms; tests that need cancel pass confirm: false.
        private static Mock<IDialogService> NewDialog(bool confirm = true)
        {
            var dialog = new Mock<IDialogService>();
            dialog.Setup(d => d.ShowConfirmation(
                It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<string>(), It.IsAny<bool>()))
                .ReturnsAsync(confirm);
            return dialog;
        }
    }
}
