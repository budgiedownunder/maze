using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the Forgot Password flow. Verifies the email-format gate,
    /// the anti-enumeration success transition, the transport-failure
    /// retry path, and the back-to-sign-in navigation.
    /// </summary>
    public class ForgotPasswordViewModelTests
    {
        private static (ForgotPasswordViewModel vm, Mock<IAuthService> auth, Mock<INavigationService> nav)
            BuildVm(bool emailEnabled = true)
        {
            var auth = new Mock<IAuthService>();
            var nav = new Mock<INavigationService>();
            var features = new Mock<IAppFeaturesService>();
            features.SetupGet(f => f.Features).Returns(new AppFeatures { EmailEnabled = emailEnabled });
            var vm = new ForgotPasswordViewModel(auth.Object, features.Object, nav.Object);
            return (vm, auth, nav);
        }

        // ---- CanSubmit -------------------------------------------------------

        [Fact]
        public void Submit_DisabledWhenEmailEmpty()
        {
            var (vm, _, _) = BuildVm();
            Assert.False(vm.SubmitCommand.CanExecute(null));
        }

        [Fact]
        public void Submit_DisabledWhileBusy()
        {
            var (vm, _, _) = BuildVm();
            vm.Email = "alice@example.com";
            vm.IsBusy = true;
            Assert.False(vm.SubmitCommand.CanExecute(null));
        }

        // ---- Submit (request endpoint) --------------------------------------

        [Fact]
        public async Task Submit_InvalidEmail_RejectsBeforeApiCall()
        {
            var (vm, auth, _) = BuildVm();
            vm.Email = "not-an-email";

            await vm.SubmitCommand.ExecuteAsync(null);

            Assert.Contains("valid email", vm.ErrorMessage);
            Assert.False(vm.Submitted);
            auth.Verify(a => a.RequestPasswordResetAsync(It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task Submit_HappyPath_FlipsSubmittedAndCallsApiOnce()
        {
            var (vm, auth, _) = BuildVm();
            vm.Email = "alice@example.com";

            await vm.SubmitCommand.ExecuteAsync(null);

            auth.Verify(a => a.RequestPasswordResetAsync("alice@example.com"), Times.Once);
            Assert.True(vm.Submitted);
            Assert.Equal("", vm.ErrorMessage);
        }

        [Fact]
        public async Task Submit_AntiEnumeration_UnknownEmailStillFlipsSubmitted()
        {
            // The wrapper returns 200 unconditionally on the server side, so
            // a Moq setup that simply completes mirrors both the
            // known-email and unknown-email paths from the VM's perspective.
            var (vm, auth, _) = BuildVm();
            auth.Setup(a => a.RequestPasswordResetAsync(It.IsAny<string>())).Returns(Task.CompletedTask);
            vm.Email = "stranger@example.com";

            await vm.SubmitCommand.ExecuteAsync(null);

            Assert.True(vm.Submitted);
            Assert.Equal("", vm.ErrorMessage);
        }

        [Fact]
        public async Task Submit_TransportFailure_SetsErrorAndKeepsSubmittedFalse()
        {
            var (vm, auth, _) = BuildVm();
            auth.Setup(a => a.RequestPasswordResetAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("network down"));
            vm.Email = "alice@example.com";

            await vm.SubmitCommand.ExecuteAsync(null);

            Assert.False(vm.Submitted);
            Assert.Contains("Could not send", vm.ErrorMessage);
        }

        [Fact]
        public async Task Submit_AfterTransportFailure_RetryCanSucceed()
        {
            var (vm, auth, _) = BuildVm();
            auth.SetupSequence(a => a.RequestPasswordResetAsync(It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("transient"))
                .Returns(Task.CompletedTask);
            vm.Email = "alice@example.com";

            await vm.SubmitCommand.ExecuteAsync(null);
            Assert.False(vm.Submitted);
            Assert.NotEqual("", vm.ErrorMessage);

            await vm.SubmitCommand.ExecuteAsync(null);

            Assert.True(vm.Submitted);
            Assert.Equal("", vm.ErrorMessage);
        }

        // ---- BackToSignIn ---------------------------------------------------

        [Fact]
        public async Task BackToSignIn_NavigatesBack()
        {
            var (vm, _, nav) = BuildVm();

            await vm.BackToSignInCommand.ExecuteAsync(null);

            nav.Verify(n => n.GoBackAsync(), Times.Once);
        }

        // ---- Email-disabled branch ------------------------------------------

        [Fact]
        public void EmailDisabled_ShowsUnavailableAndHidesForm()
        {
            var (vm, _, _) = BuildVm(emailEnabled: false);

            Assert.False(vm.EmailEnabled);
            Assert.True(vm.ShowUnavailable);
            Assert.False(vm.ShowForm);
            Assert.False(vm.ShowSuccess);
        }

        [Fact]
        public void EmailEnabled_StartsOnFormState()
        {
            var (vm, _, _) = BuildVm(emailEnabled: true);

            Assert.True(vm.EmailEnabled);
            Assert.True(vm.ShowForm);
            Assert.False(vm.ShowSuccess);
            Assert.False(vm.ShowUnavailable);
        }

        [Fact]
        public async Task EmailEnabled_FlipsToSuccessAfterSubmit()
        {
            var (vm, _, _) = BuildVm(emailEnabled: true);
            vm.Email = "alice@example.com";

            await vm.SubmitCommand.ExecuteAsync(null);

            Assert.False(vm.ShowForm);
            Assert.True(vm.ShowSuccess);
            Assert.False(vm.ShowUnavailable);
        }
    }
}
