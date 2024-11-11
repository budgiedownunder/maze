using Maze.Maui.App.Controls.Pointer;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class BorderBox : BoxView
    {
        public delegate void PanUpdatedHandler(BorderBox sender, PanUpdatedEventArgs e);
        public event PanUpdatedHandler? PanUpdated;

        private Icon HoverIcon { get; }

        public BorderBox(Icon hoverIcon)
        {
            HoverIcon = hoverIcon;
            RegisterEventHandlers();
        }

        private void RegisterEventHandlers()
        {
            RegisterPointerEventHandlers();
            RegisterPanEventHandlers();
        }

        private void RegisterPointerEventHandlers()
        {
            var pointerGestureRecognizer = new PointerGestureRecognizer();
            pointerGestureRecognizer.PointerEntered += (s, e) =>
            {
                OnPointerEntered();
            };
            pointerGestureRecognizer.PointerExited += (s, e) =>
            {
                OnPointerExited();
            };
            GestureRecognizers.Add(pointerGestureRecognizer);
        }

        private void OnPointerEntered()
        {
            Pointer.Pointer.SetCursor(this, HoverIcon);
        }

        private void OnPointerExited()
        {
            Pointer.Pointer.SetCursor(this, Icon.Arrow);
        }

        private void RegisterPanEventHandlers()
        {
            var panGestureRecognizer = new PanGestureRecognizer();
            panGestureRecognizer.PanUpdated += (s, e) =>
            {
                OnPanUpdated(e);
            };
            GestureRecognizers.Add(panGestureRecognizer);
        }

        private void OnPanUpdated(PanUpdatedEventArgs e)
        {
            if (PanUpdated != null)
            {
                PanUpdated.Invoke(this, e);
            }
        }

    }
}
