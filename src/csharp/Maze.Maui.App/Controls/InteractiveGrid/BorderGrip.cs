
using Maze.Maui.App.Controls.Pointer;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    /// <summary>
    /// The `BorderGrip` class represents a circular grip attached to a border
    /// </summary>
    public class BorderGrip : BoxView
    {
        // Private property data
        private double size;

        /// <summary>
        /// Pan updated delegate handler
        /// </summary>
        /// <param name="sender">The border grip sender</param>
        /// <param name="e">Pan updated event arguments</param>
        public delegate void PanUpdatedHandler(BorderGrip sender, PanUpdatedEventArgs e);
        /// <summary>
        /// The registered pan updated event handler (if any)
        /// </summary>
        /// <returns>Pan updated event handler</returns>
        public event PanUpdatedHandler? PanUpdated;
        /// <summary>
        /// The icon to be displayed when the mouse pointer hovers over the object
        /// </summary>
        /// <returns>Hover icon</returns>
        public Icon HoverIcon { get; set; }
        /// <summary>
        /// The diameter of the grip
        /// </summary>
        /// <returns>Diameter</returns>
        public double Diameter
        {
            get => size;
            set
            {
                size = value;
                HeightRequest = Diameter;
                WidthRequest = Diameter;
                CornerRadius = Diameter / 2.0;
            }
        }
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="hoverIcon">The icon to be displayed when the mouse pointer hovers over the object</param>
        /// <param name="diameter">The diameter of the grip</param>
        public BorderGrip(Icon hoverIcon, double diameter)
        {
            HoverIcon = hoverIcon;
            Diameter = diameter;
            RegisterEventHandlers();
        }
        /// <summary>
        /// Registers internal event handlers
        /// </summary>
        private void RegisterEventHandlers()
        {
            RegisterPointerEventHandlers();
            RegisterPanEventHandlers();
        }
        /// <summary>
        /// Registers internal pointer event handlers
        /// </summary>
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
        /// <summary>
        /// Handles the pointer entered event, setting the cursor to the hover icon associated
        /// with the object
        /// </summary>
        private void OnPointerEntered()
        {
            Pointer.Pointer.SetCursor(this, HoverIcon);
        }
        /// <summary>
        /// Handles the pointer exited event, resetting the cursor to arrow icon
        /// </summary>
        private void OnPointerExited()
        {
            Pointer.Pointer.SetCursor(this, Icon.Arrow);
        }
        /// <summary>
        /// Registers internal pan event handlers
        /// </summary>
        private void RegisterPanEventHandlers()
        {
            var panGestureRecognizer = new PanGestureRecognizer();
            panGestureRecognizer.PanUpdated += (s, e) =>
            {
                OnPanUpdated(e);
            };
            GestureRecognizers.Add(panGestureRecognizer);
        }
        /// <summary>
        /// Handles the pan updated event, triggering any registed PanUpdated handler
        /// </summary>
        private void OnPanUpdated(PanUpdatedEventArgs e)
        {
            if (PanUpdated != null)
            {
                PanUpdated.Invoke(this, e);
            }
        }

    }
}
