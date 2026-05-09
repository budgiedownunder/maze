// Stub of the MAUI ContentPage. LoginViewModel and SignUpViewModel use
// `nameof(AccountPage)` to build a Shell route after a brand-new OAuth
// sign-up, so the symbol must be resolvable when those ViewModels are
// file-linked into the non-MAUI test host. `nameof` evaluates at compile
// time and only requires the type's name — an empty class in the same
// namespace is enough.
namespace Maze.Maui.App.Views
{
    internal sealed class AccountPage { }
}
