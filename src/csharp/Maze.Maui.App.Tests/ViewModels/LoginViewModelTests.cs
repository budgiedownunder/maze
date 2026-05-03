using System.Net;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Maze.Maui.App.Views;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the credentials sign-in flow, the OAuth sign-in flow,
    /// the session-restore lifecycle, and the OAuth provider list sync.
    /// All Shell.Current navigation is verified via INavigationService;
    /// AccountViewModel is constructed with the same mocks but its
    /// behaviour is exercised in <see cref="AccountViewModelTests"/>.
    /// </summary>
    public class LoginViewModelTests
    {
        private static AccountViewModel BuildAccountVm()
            => new(new Mock<IAuthService>().Object, new Mock<IDialogService>().Object, new Mock<INavigationService>().Object);

        private static (LoginViewModel vm, Mock<IAuthService> auth, Mock<IAppFeaturesService> features, Mock<INavigationService> nav, AccountViewModel account)
            BuildVm(AppFeatures? appFeatures = null)
        {
            var auth = new Mock<IAuthService>();
            var features = new Mock<IAppFeaturesService>();
            features.SetupGet(f => f.Features).Returns(appFeatures ?? new AppFeatures());
            features.Setup(f => f.RefreshAsync()).Returns(Task.CompletedTask);
            var nav = new Mock<INavigationService>();
            var account = BuildAccountVm();
            var vm = new LoginViewModel(auth.Object, features.Object, nav.Object, account);
            return (vm, auth, features, nav, account);
        }

        // ---- CanSignIn -------------------------------------------------------

        [Fact]
        public void CanSignIn_FalseUntilEmailAndPasswordSet()
        {
            var (vm, _, _, _, _) = BuildVm();
            Assert.False(vm.SignInCommand.CanExecute(null));
            vm.Email = "alice@example.com";
            Assert.False(vm.SignInCommand.CanExecute(null));
            vm.Password = "Pass1!";
            Assert.True(vm.SignInCommand.CanExecute(null));
        }

        [Fact]
        public void CanSignIn_FalseWhileBusy()
        {
            var (vm, _, _, _, _) = BuildVm();
            vm.Email = "alice@example.com";
            vm.Password = "Pass1!";
            vm.IsBusy = true;
            Assert.False(vm.SignInCommand.CanExecute(null));
        }

        // ---- SignIn (credentials) -------------------------------------------

        [Fact]
        public async Task SignIn_InvalidEmail_RejectsBeforeApiCall()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            vm.Email = "not-an-email";
            vm.Password = "Pass1!";

            await vm.SignInCommand.ExecuteAsync(null);

            Assert.Contains("valid email", vm.ErrorMessage);
            auth.Verify(a => a.SignInAsync(It.IsAny<string>(), It.IsAny<string>()), Times.Never);
            nav.VerifyNoOtherCalls();
        }

        [Fact]
        public async Task SignIn_HappyPath_CallsAuthAndNavigatesToMainPage()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            vm.Email = "alice@example.com";
            vm.Password = "Pass1!";

            await vm.SignInCommand.ExecuteAsync(null);

            auth.Verify(a => a.SignInAsync("alice@example.com", "Pass1!"), Times.Once);
            nav.Verify(n => n.GoToRootAsync("//MainPage"), Times.Once);
        }

        [Fact]
        public async Task SignIn_401_ShowsInvalidCredentialsAndDoesNotNavigate()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            auth.Setup(a => a.SignInAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("unauthorized", null, HttpStatusCode.Unauthorized));
            vm.Email = "alice@example.com";
            vm.Password = "WrongPass1!";

            await vm.SignInCommand.ExecuteAsync(null);

            Assert.Contains("Invalid email or password", vm.ErrorMessage);
            nav.Verify(n => n.GoToRootAsync(It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task SignIn_GenericException_ShowsGenericFailureAndDoesNotNavigate()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            auth.Setup(a => a.SignInAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new InvalidOperationException("network down"));
            vm.Email = "alice@example.com";
            vm.Password = "Pass1!";

            await vm.SignInCommand.ExecuteAsync(null);

            Assert.Contains("Sign in failed", vm.ErrorMessage);
            nav.Verify(n => n.GoToRootAsync(It.IsAny<string>()), Times.Never);
        }

        // ---- ToggleShowPassword + GoToSignUp --------------------------------

        [Fact]
        public void ToggleShowPassword_FlipsFlag()
        {
            var (vm, _, _, _, _) = BuildVm();
            Assert.False(vm.ShowPassword);
            vm.ToggleShowPasswordCommand.Execute(null);
            Assert.True(vm.ShowPassword);
            vm.ToggleShowPasswordCommand.Execute(null);
            Assert.False(vm.ShowPassword);
        }

        [Fact]
        public async Task GoToSignUp_NavigatesToSignUpPage()
        {
            var (vm, _, _, nav, _) = BuildVm();
            await vm.GoToSignUpCommand.ExecuteAsync(null);
            nav.Verify(n => n.GoToAsync(nameof(SignUpPage), null), Times.Once);
        }

        // ---- TryRestoreSessionAsync -----------------------------------------

        [Fact]
        public async Task TryRestoreSession_NotAuthenticated_ReturnsFalse()
        {
            var (vm, auth, _, _, _) = BuildVm();
            auth.Setup(a => a.IsAuthenticatedAsync()).ReturnsAsync(false);

            var result = await vm.TryRestoreSessionAsync();

            Assert.False(result);
            auth.Verify(a => a.GetMyProfileAsync(), Times.Never);
        }

        [Fact]
        public async Task TryRestoreSession_HappyPath_ReturnsTrue()
        {
            var (vm, auth, _, _, _) = BuildVm();
            auth.Setup(a => a.IsAuthenticatedAsync()).ReturnsAsync(true);
            auth.Setup(a => a.GetMyProfileAsync()).ReturnsAsync(new UserProfile());

            var result = await vm.TryRestoreSessionAsync();

            Assert.True(result);
        }

        [Fact]
        public async Task TryRestoreSession_401_SignsOutAndReportsExpiredSession()
        {
            var (vm, auth, _, _, _) = BuildVm();
            auth.Setup(a => a.IsAuthenticatedAsync()).ReturnsAsync(true);
            auth.Setup(a => a.GetMyProfileAsync())
                .ThrowsAsync(new HttpRequestException("unauthorized", null, HttpStatusCode.Unauthorized));

            var result = await vm.TryRestoreSessionAsync();

            Assert.False(result);
            auth.Verify(a => a.SignOutAsync(), Times.Once);
            Assert.Contains("session has expired", vm.ErrorMessage);
        }

        [Fact]
        public async Task TryRestoreSession_RefreshesFeaturesAndPropagatesAllowSignUp()
        {
            var (vm, auth, features, _, _) = BuildVm(new AppFeatures { AllowSignUp = false });
            auth.Setup(a => a.IsAuthenticatedAsync()).ReturnsAsync(false);

            await vm.TryRestoreSessionAsync();

            features.Verify(f => f.RefreshAsync(), Times.Once);
            Assert.False(vm.AllowSignUp);
        }

        // ---- OAuth provider sync --------------------------------------------

        [Fact]
        public async Task TryRestoreSession_PopulatesOAuthProviders()
        {
            var providers = new List<OAuthProviderPublic>
            {
                new() { Name = "google", DisplayName = "Google" },
                new() { Name = "github", DisplayName = "GitHub" },
            };
            var (vm, auth, _, _, _) = BuildVm(new AppFeatures { OAuthProviders = providers });
            auth.Setup(a => a.IsAuthenticatedAsync()).ReturnsAsync(false);
            // Track HasOAuthProviders change notifications.
            int hasOAuthChanges = 0;
            vm.PropertyChanged += (_, e) =>
            {
                if (e.PropertyName == nameof(LoginViewModel.HasOAuthProviders)) hasOAuthChanges++;
            };

            await vm.TryRestoreSessionAsync();

            Assert.Equal(2, vm.OAuthProviders.Count);
            Assert.True(vm.HasOAuthProviders);
            Assert.True(hasOAuthChanges >= 1);
        }

        // ---- SignInWithOAuth -------------------------------------------------

        [Fact]
        public async Task SignInWithOAuth_BlankProvider_DoesNothing()
        {
            var (vm, auth, _, nav, _) = BuildVm();

            await vm.SignInWithOAuthCommand.ExecuteAsync("");

            auth.Verify(a => a.SignInWithOAuthAsync(It.IsAny<string>()), Times.Never);
            nav.VerifyNoOtherCalls();
        }

        [Fact]
        public async Task SignInWithOAuth_NewUser_FlipsAccountWelcomeModeAndNavigates()
        {
            var (vm, auth, _, nav, account) = BuildVm();
            auth.Setup(a => a.SignInWithOAuthAsync("google"))
                .ReturnsAsync(new OAuthSignInResult { IsNewUser = true });

            await vm.SignInWithOAuthCommand.ExecuteAsync("google");

            Assert.True(account.IsWelcomeMode);
            nav.Verify(n => n.GoToRootAsync("//MainPage"), Times.Once);
        }

        [Fact]
        public async Task SignInWithOAuth_ExistingUser_DoesNotFlipWelcomeModeAndNavigates()
        {
            var (vm, auth, _, nav, account) = BuildVm();
            auth.Setup(a => a.SignInWithOAuthAsync("google"))
                .ReturnsAsync(new OAuthSignInResult { IsNewUser = false });

            await vm.SignInWithOAuthCommand.ExecuteAsync("google");

            Assert.False(account.IsWelcomeMode);
            nav.Verify(n => n.GoToRootAsync("//MainPage"), Times.Once);
        }

        [Fact]
        public async Task SignInWithOAuth_FlowFailure_MapsReasonToFriendlyMessage()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            auth.Setup(a => a.SignInWithOAuthAsync("google"))
                .ThrowsAsync(new OAuthFlowFailedException("signup_disabled"));

            await vm.SignInWithOAuthCommand.ExecuteAsync("google");

            Assert.Contains("Sign-up is disabled", vm.ErrorMessage);
            nav.Verify(n => n.GoToRootAsync(It.IsAny<string>()), Times.Never);
        }

        [Fact]
        public async Task SignInWithOAuth_Timeout_ShowsCancelOrTimeoutMessage()
        {
            var (vm, auth, _, _, _) = BuildVm();
            auth.Setup(a => a.SignInWithOAuthAsync("google"))
                .ThrowsAsync(new TimeoutException());

            await vm.SignInWithOAuthCommand.ExecuteAsync("google");

            Assert.Contains("cancelled or did not complete", vm.ErrorMessage);
        }

        [Fact]
        public async Task SignInWithOAuth_TaskCancelled_DoesNotShowError()
        {
            var (vm, auth, _, _, _) = BuildVm();
            auth.Setup(a => a.SignInWithOAuthAsync("google"))
                .ThrowsAsync(new TaskCanceledException());

            await vm.SignInWithOAuthCommand.ExecuteAsync("google");

            Assert.Equal("", vm.ErrorMessage);
        }

        // ---- RefreshOAuthProviderItems --------------------------------------

        [Fact]
        public void RefreshOAuthProviderItems_EmptyCollection_NoOp()
        {
            var (vm, _, _, _, _) = BuildVm();
            // Should not throw and should leave the empty collection empty.
            vm.RefreshOAuthProviderItems();
            Assert.Empty(vm.OAuthProviders);
        }
    }
}
