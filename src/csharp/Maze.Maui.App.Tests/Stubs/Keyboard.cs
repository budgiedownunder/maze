// Stub of MAUI's Microsoft.Maui.Controls.Keyboard. The real type is a
// runtime descriptor for soft-keyboard variants used by Entry controls;
// it appears in IDialogService.DisplayPrompt's signature but the test
// host never invokes that method (it's mocked). Declaring a same-named
// class in Maze.Maui.App.Services lets the file-linked IDialogService.cs
// resolve `Keyboard?` without pulling in MAUI runtime references.
namespace Maze.Maui.App.Services
{
    public sealed class Keyboard { }
}
