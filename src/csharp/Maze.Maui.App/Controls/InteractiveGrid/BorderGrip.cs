
using Maze.Maui.App.Controls.Pointer;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class BorderGrip : BoxView
    {
        public delegate void PanUpdatedHandler(BorderGrip sender, PanUpdatedEventArgs e);
        public event PanUpdatedHandler? PanUpdated;

        private double size;
        private Icon HoverIcon { get; }

        public double Size
        {
            get => size;
            set
            {
                size = value;
                HeightRequest = Size;
                WidthRequest = Size;
                CornerRadius = Size / 2.0;
            }
        }
        public BorderGrip(Icon hoverIcon, double size)
        {
            HoverIcon = hoverIcon;
            Size = size;
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
