// Stub of the MAUI ContentPage. AccountViewModel uses
// `nameof(ChangePasswordPage)` to build a Shell route, so the symbol
// must be resolvable when AccountViewModel.cs is file-linked into the
// non-MAUI test host. `nameof` evaluates at compile time and only
// requires the type's name — an empty class in the same namespace is
// enough.
namespace Maze.Maui.App.Views
{
    internal sealed class ChangePasswordPage { }
}
