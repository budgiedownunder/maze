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
            var vm = new EmailAddressesViewModel(auth.Object);

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
            var vm = new EmailAddressesViewModel(auth.Object);

            await vm.LoadEmailsCommand.ExecuteAsync(null);

            Assert.Empty(vm.Emails);
            Assert.Contains("Failed to load", vm.ErrorMessage);
        }

        // ---- AddEmail ------------------------------------------------------

        [Fact]
        public void CanAddEmail_FalseUntilFormatValid()
        {
            var auth = new Mock<IAuthService>();
            var vm = new EmailAddressesViewModel(auth.Object);

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
            var vm = new EmailAddressesViewModel(auth.Object);
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
            var vm = new EmailAddressesViewModel(auth.Object)
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
            var vm = new EmailAddressesViewModel(auth.Object)
            {
                NewEmail = "weird@example.com"
            };

            await vm.AddEmailCommand.ExecuteAsync(null);

            Assert.Contains("Email format is invalid", vm.ErrorMessage);
        }

        // ---- RemoveEmail ---------------------------------------------------

        [Fact]
        public async Task RemoveEmail_RemovesRowOnSuccess()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.RemoveEmailAsync("second@example.com"))
                .ReturnsAsync(new List<UserEmail> { PrimaryRow });
            var vm = SeedTwoRowVm(auth);
            var secondRow = vm.Emails.First(r => r.Email == "second@example.com");

            await vm.RemoveEmailCommand.ExecuteAsync(secondRow);

            Assert.Single(vm.Emails);
            Assert.Equal("primary@example.com", vm.Emails[0].Email);
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
        public async Task VerifyEmail_Surfaces501AsNotYetAvailable()
        {
            var auth = new Mock<IAuthService>();
            auth.Setup(s => s.VerifyEmailAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("stub", null, HttpStatusCode.NotImplemented));
            var vm = SeedTwoRowVm(auth);
            var row = vm.Emails.First();

            await vm.VerifyEmailCommand.ExecuteAsync(row);

            Assert.Contains("not yet available", vm.ErrorMessage);
        }

        // ---- helpers -------------------------------------------------------

        private static EmailAddressesViewModel SeedTwoRowVm(Mock<IAuthService> auth)
        {
            var vm = new EmailAddressesViewModel(auth.Object);
            vm.Emails.Add(new EmailRowViewModel { Email = "primary@example.com", IsPrimary = true, Verified = true });
            vm.Emails.Add(new EmailRowViewModel { Email = "second@example.com", IsPrimary = false, Verified = true });
            return vm;
        }
    }
}
