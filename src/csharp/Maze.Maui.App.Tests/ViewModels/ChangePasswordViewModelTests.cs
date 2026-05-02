using System.Net;
using CommunityToolkit.Mvvm.Messaging;
using Maze.Maui.App.Messages;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the password Change/Set branching, the messenger send,
    /// and the back-navigation step. The Shell.Current dependency is
    /// abstracted through INavigationService so these run inside the
    /// non-MAUI test host.
    /// </summary>
    public class ChangePasswordViewModelTests
    {
        private const string ValidNew = "NewPass1!";

        private static (ChangePasswordViewModel vm, Mock<IAuthService> auth, Mock<INavigationService> nav)
            BuildVm(bool hasPassword)
        {
            var auth = new Mock<IAuthService>();
            var nav = new Mock<INavigationService>();
            var vm = new ChangePasswordViewModel(auth.Object, nav.Object) { HasPassword = hasPassword };
            return (vm, auth, nav);
        }

        // ---- Change variant -------------------------------------------------

        [Fact]
        public void ChangeVariant_HeadingAndButtonText()
        {
            var (vm, _, _) = BuildVm(hasPassword: true);
            Assert.Equal("Change Password", vm.HeadingText);
            Assert.Equal("Change Password", vm.SubmitText);
            Assert.Equal("Changing...", vm.SubmitBusyText);
        }

        [Fact]
        public void ChangeVariant_RequiresAllThreeFields()
        {
            var (vm, _, _) = BuildVm(hasPassword: true);
            // No fields filled → command can't execute
            Assert.False(vm.ChangePasswordCommand.CanExecute(null));
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;
            // Still missing CurrentPassword
            Assert.False(vm.ChangePasswordCommand.CanExecute(null));
            vm.CurrentPassword = "OldPass1!";
            Assert.True(vm.ChangePasswordCommand.CanExecute(null));
        }

        [Fact]
        public async Task ChangeVariant_CallsChangePasswordAsyncAndNavigatesBack()
        {
            var (vm, auth, nav) = BuildVm(hasPassword: true);
            vm.CurrentPassword = "OldPass1!";
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;

            await vm.ChangePasswordCommand.ExecuteAsync(null);

            auth.Verify(a => a.ChangePasswordAsync("OldPass1!", ValidNew), Times.Once);
            auth.Verify(a => a.SetInitialPasswordAsync(It.IsAny<string>()), Times.Never);
            nav.Verify(n => n.GoBackAsync(), Times.Once);
        }

        [Fact]
        public async Task ChangeVariant_Surfaces401AsCurrentPasswordIncorrect()
        {
            var (vm, auth, nav) = BuildVm(hasPassword: true);
            auth.Setup(a => a.ChangePasswordAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("unauthorized", null, HttpStatusCode.Unauthorized));
            vm.CurrentPassword = "WrongPass1!";
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;

            await vm.ChangePasswordCommand.ExecuteAsync(null);

            Assert.Contains("Current password is incorrect", vm.ErrorMessage);
            nav.Verify(n => n.GoBackAsync(), Times.Never);
        }

        // ---- Set variant ----------------------------------------------------

        [Fact]
        public void SetVariant_HeadingAndButtonText()
        {
            var (vm, _, _) = BuildVm(hasPassword: false);
            Assert.Equal("Set Password", vm.HeadingText);
            Assert.Equal("Set Password", vm.SubmitText);
            Assert.Equal("Setting...", vm.SubmitBusyText);
        }

        [Fact]
        public void SetVariant_DoesNotRequireCurrentPassword()
        {
            var (vm, _, _) = BuildVm(hasPassword: false);
            Assert.False(vm.ChangePasswordCommand.CanExecute(null));
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;
            // No CurrentPassword needed in Set variant
            Assert.True(vm.ChangePasswordCommand.CanExecute(null));
        }

        [Fact]
        public async Task SetVariant_CallsSetInitialPasswordAsyncAndNavigatesBack()
        {
            var (vm, auth, nav) = BuildVm(hasPassword: false);
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;

            await vm.ChangePasswordCommand.ExecuteAsync(null);

            auth.Verify(a => a.SetInitialPasswordAsync(ValidNew), Times.Once);
            auth.Verify(a => a.ChangePasswordAsync(It.IsAny<string>(), It.IsAny<string>()), Times.Never);
            nav.Verify(n => n.GoBackAsync(), Times.Once);
        }

        [Fact]
        public async Task SetVariant_DoesNotSurface401AsCurrentPasswordError()
        {
            // The Set variant has no current password — a 401 (which shouldn't
            // happen on this path anyway) must NOT be reported as
            // "Current password is incorrect".
            var (vm, auth, _) = BuildVm(hasPassword: false);
            auth.Setup(a => a.SetInitialPasswordAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("unauthorized", null, HttpStatusCode.Unauthorized));
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;

            await vm.ChangePasswordCommand.ExecuteAsync(null);

            Assert.Contains("Failed to set password", vm.ErrorMessage);
            Assert.DoesNotContain("Current password", vm.ErrorMessage);
        }

        // ---- Validation (shared) -------------------------------------------

        [Fact]
        public async Task MismatchedConfirm_Rejects_BeforeApiCall()
        {
            var (vm, auth, nav) = BuildVm(hasPassword: true);
            vm.CurrentPassword = "OldPass1!";
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = "Different1!";

            await vm.ChangePasswordCommand.ExecuteAsync(null);

            Assert.Contains("do not match", vm.ErrorMessage);
            auth.VerifyNoOtherCalls();
            nav.VerifyNoOtherCalls();
        }

        [Fact]
        public async Task WeakNewPassword_Rejects_BeforeApiCall()
        {
            var (vm, auth, nav) = BuildVm(hasPassword: false);
            vm.NewPassword = "weak";
            vm.ConfirmNewPassword = "weak";

            await vm.ChangePasswordCommand.ExecuteAsync(null);

            Assert.Contains("at least 8 characters", vm.ErrorMessage);
            auth.VerifyNoOtherCalls();
            nav.VerifyNoOtherCalls();
        }

        // ---- PasswordSetMessage --------------------------------------------

        [Fact]
        public async Task SuccessfulChange_FiresPasswordSetMessage()
        {
            var (vm, _, _) = BuildVm(hasPassword: true);
            vm.CurrentPassword = "OldPass1!";
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;
            int received = 0;
            object recipient = new();
            WeakReferenceMessenger.Default.Register<PasswordSetMessage>(recipient, (_, _) => received++);
            try
            {
                await vm.ChangePasswordCommand.ExecuteAsync(null);
            }
            finally
            {
                WeakReferenceMessenger.Default.Unregister<PasswordSetMessage>(recipient);
            }

            Assert.Equal(1, received);
        }

        [Fact]
        public async Task SuccessfulSet_FiresPasswordSetMessage()
        {
            var (vm, _, _) = BuildVm(hasPassword: false);
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;
            int received = 0;
            object recipient = new();
            WeakReferenceMessenger.Default.Register<PasswordSetMessage>(recipient, (_, _) => received++);
            try
            {
                await vm.ChangePasswordCommand.ExecuteAsync(null);
            }
            finally
            {
                WeakReferenceMessenger.Default.Unregister<PasswordSetMessage>(recipient);
            }

            Assert.Equal(1, received);
        }

        [Fact]
        public async Task FailedChange_DoesNotFirePasswordSetMessage()
        {
            var (vm, auth, _) = BuildVm(hasPassword: true);
            auth.Setup(a => a.ChangePasswordAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("unauthorized", null, HttpStatusCode.Unauthorized));
            vm.CurrentPassword = "WrongPass1!";
            vm.NewPassword = ValidNew;
            vm.ConfirmNewPassword = ValidNew;
            int received = 0;
            object recipient = new();
            WeakReferenceMessenger.Default.Register<PasswordSetMessage>(recipient, (_, _) => received++);
            try
            {
                await vm.ChangePasswordCommand.ExecuteAsync(null);
            }
            finally
            {
                WeakReferenceMessenger.Default.Unregister<PasswordSetMessage>(recipient);
            }

            Assert.Equal(0, received);
        }

        // ---- GoBack command -------------------------------------------------

        [Fact]
        public async Task GoBack_DelegatesToNavigationService()
        {
            var (vm, _, nav) = BuildVm(hasPassword: true);

            await vm.GoBackCommand.ExecuteAsync(null);

            nav.Verify(n => n.GoBackAsync(), Times.Once);
        }
    }
}
