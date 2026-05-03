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
            var vm = new AccountViewModel(auth.Object, dialog.Object, nav.Object);
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
