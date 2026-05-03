using System.Net;
using Maze.Maui.App.Services;
using Maze.Maui.App.ViewModels;
using Moq;
using Xunit;

namespace Maze.Maui.App.Tests.ViewModels
{
    /// <summary>
    /// Tests for the sign-up form: CanSignUp guard, password match,
    /// password complexity rules, server 409 handling, the
    /// error-message-clearing partial property handlers, and the OAuth
    /// fallback (which the page renders alongside the credentials form).
    /// </summary>
    public class SignUpViewModelTests
    {
        private const string ValidPassword = "Pass1!aB";

        private static AccountViewModel BuildAccountVm()
            => new(new Mock<IAuthService>().Object, new Mock<IDialogService>().Object, new Mock<INavigationService>().Object);

        private static (SignUpViewModel vm, Mock<IAuthService> auth, Mock<IAppFeaturesService> features, Mock<INavigationService> nav, AccountViewModel account)
            BuildVm()
        {
            var auth = new Mock<IAuthService>();
            var features = new Mock<IAppFeaturesService>();
            features.SetupGet(f => f.Features).Returns(new AppFeatures());
            features.Setup(f => f.RefreshAsync()).Returns(Task.CompletedTask);
            var nav = new Mock<INavigationService>();
            var account = BuildAccountVm();
            var vm = new SignUpViewModel(auth.Object, features.Object, nav.Object, account);
            return (vm, auth, features, nav, account);
        }

        // ---- CanSignUp guards ------------------------------------------------

        [Fact]
        public void CanSignUp_FalseUntilAllThreeFieldsSet()
        {
            var (vm, _, _, _, _) = BuildVm();
            Assert.False(vm.SignUpCommand.CanExecute(null));
            vm.Email = "alice@example.com";
            Assert.False(vm.SignUpCommand.CanExecute(null));
            vm.Password = ValidPassword;
            Assert.False(vm.SignUpCommand.CanExecute(null));
            vm.ConfirmPassword = ValidPassword;
            Assert.True(vm.SignUpCommand.CanExecute(null));
        }

        [Fact]
        public void CanSignUp_FalseWhileBusy()
        {
            var (vm, _, _, _, _) = BuildVm();
            vm.Email = "alice@example.com";
            vm.Password = ValidPassword;
            vm.ConfirmPassword = ValidPassword;
            vm.IsBusy = true;
            Assert.False(vm.SignUpCommand.CanExecute(null));
        }

        // ---- Validation: passwords match ------------------------------------

        [Fact]
        public async Task SignUp_PasswordsDoNotMatch_RejectsBeforeApiCall()
        {
            var (vm, auth, _, _, _) = BuildVm();
            vm.Email = "alice@example.com";
            vm.Password = ValidPassword;
            vm.ConfirmPassword = "Different1!";

            await vm.SignUpCommand.ExecuteAsync(null);

            Assert.Contains("do not match", vm.ErrorMessage);
            auth.Verify(a => a.SignUpAsync(It.IsAny<string>(), It.IsAny<string>()), Times.Never);
        }

        // ---- Validation: complexity (one Theory per rule) -------------------

        [Theory]
        [InlineData("Sh0rt!")]            // < 8 chars
        [InlineData("alllower1!")]         // no uppercase
        [InlineData("ALLUPPER1!")]         // no lowercase
        [InlineData("NoDigits!!")]         // no digit
        [InlineData("NoSymbol1A")]         // no special character
        public async Task SignUp_PasswordFailsComplexity_RejectsBeforeApiCall(string password)
        {
            var (vm, auth, _, _, _) = BuildVm();
            vm.Email = "alice@example.com";
            vm.Password = password;
            vm.ConfirmPassword = password;

            await vm.SignUpCommand.ExecuteAsync(null);

            Assert.Contains("at least 8 characters", vm.ErrorMessage);
            auth.Verify(a => a.SignUpAsync(It.IsAny<string>(), It.IsAny<string>()), Times.Never);
        }

        // ---- Happy path + back-navigation -----------------------------------

        [Fact]
        public async Task SignUp_HappyPath_CallsAuthAndNavigatesBack()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            vm.Email = "alice@example.com";
            vm.Password = ValidPassword;
            vm.ConfirmPassword = ValidPassword;

            await vm.SignUpCommand.ExecuteAsync(null);

            auth.Verify(a => a.SignUpAsync("alice@example.com", ValidPassword), Times.Once);
            nav.Verify(n => n.GoBackAsync(), Times.Once);
        }

        // ---- Server error handling -------------------------------------------

        [Fact]
        public async Task SignUp_409_ShowsEmailAlreadyInUseAndDoesNotNavigate()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            auth.Setup(a => a.SignUpAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new HttpRequestException("conflict", null, HttpStatusCode.Conflict));
            vm.Email = "taken@example.com";
            vm.Password = ValidPassword;
            vm.ConfirmPassword = ValidPassword;

            await vm.SignUpCommand.ExecuteAsync(null);

            Assert.Contains("Email already in use", vm.ErrorMessage);
            nav.Verify(n => n.GoBackAsync(), Times.Never);
        }

        [Fact]
        public async Task SignUp_GenericException_ShowsGenericFailureAndDoesNotNavigate()
        {
            var (vm, auth, _, nav, _) = BuildVm();
            auth.Setup(a => a.SignUpAsync(It.IsAny<string>(), It.IsAny<string>()))
                .ThrowsAsync(new InvalidOperationException("network down"));
            vm.Email = "alice@example.com";
            vm.Password = ValidPassword;
            vm.ConfirmPassword = ValidPassword;

            await vm.SignUpCommand.ExecuteAsync(null);

            Assert.Contains("Sign up failed", vm.ErrorMessage);
            nav.Verify(n => n.GoBackAsync(), Times.Never);
        }

        // ---- Toggle commands -------------------------------------------------

        [Fact]
        public void ToggleShowPassword_FlipsFlag()
        {
            var (vm, _, _, _, _) = BuildVm();
            Assert.False(vm.ShowPassword);
            vm.ToggleShowPasswordCommand.Execute(null);
            Assert.True(vm.ShowPassword);
        }

        [Fact]
        public void ToggleShowConfirmPassword_FlipsFlag()
        {
            var (vm, _, _, _, _) = BuildVm();
            Assert.False(vm.ShowConfirmPassword);
            vm.ToggleShowConfirmPasswordCommand.Execute(null);
            Assert.True(vm.ShowConfirmPassword);
        }

        // ---- Error-message-clearing partial handlers ------------------------

        [Fact]
        public void TypingIntoFields_ClearsExistingErrorMessage()
        {
            var (vm, _, _, _, _) = BuildVm();
            vm.ErrorMessage = "Previous error";
            vm.Email = "x";
            Assert.Equal("", vm.ErrorMessage);

            vm.ErrorMessage = "Previous error";
            vm.Password = "x";
            Assert.Equal("", vm.ErrorMessage);

            vm.ErrorMessage = "Previous error";
            vm.ConfirmPassword = "x";
            Assert.Equal("", vm.ErrorMessage);
        }

        // ---- GoBack ----------------------------------------------------------

        [Fact]
        public async Task GoBack_DelegatesToNavigationService()
        {
            var (vm, _, _, nav, _) = BuildVm();
            await vm.GoBackCommand.ExecuteAsync(null);
            nav.Verify(n => n.GoBackAsync(), Times.Once);
        }

        // ---- RefreshOAuthProvidersAsync -------------------------------------

        [Fact]
        public async Task RefreshOAuthProvidersAsync_PopulatesCollectionAndNotifies()
        {
            var providers = new List<OAuthProviderPublic>
            {
                new() { Name = "google", DisplayName = "Google" },
            };
            var auth = new Mock<IAuthService>();
            var features = new Mock<IAppFeaturesService>();
            features.SetupGet(f => f.Features).Returns(new AppFeatures { OAuthProviders = providers });
            features.Setup(f => f.RefreshAsync()).Returns(Task.CompletedTask);
            var nav = new Mock<INavigationService>();
            var vm = new SignUpViewModel(auth.Object, features.Object, nav.Object, BuildAccountVm());
            int hasOAuthChanges = 0;
            vm.PropertyChanged += (_, e) =>
            {
                if (e.PropertyName == nameof(SignUpViewModel.HasOAuthProviders)) hasOAuthChanges++;
            };

            await vm.RefreshOAuthProvidersAsync();

            Assert.Single(vm.OAuthProviders);
            Assert.True(vm.HasOAuthProviders);
            Assert.True(hasOAuthChanges >= 1);
        }
    }
}
