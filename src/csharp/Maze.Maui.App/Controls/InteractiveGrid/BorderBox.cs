using Maze.Maui.App.Controls.Pointer;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    /// <summary>
    /// The `BoxDrawable` class represents a box that can be drawn either as a solid box or as dashed
    /// box. In addition, a dashed offset can be used to create animation affects by sequentially
    /// adjusting the value that is applied to the base dash pattern when the box is redrawn.
    /// </summary>
    internal class BoxDrawable : IDrawable
    {
        /// <summary>
        /// Background color
        /// </summary>
        /// <returns>Background color</returns>
        public Color BackgroundColor { get; set; } = Colors.Transparent;
        /// <summary>
        /// Whether the box should be drawn with a dashed pattern
        /// </summary>
        /// <returns>Boolean</returns>
        public bool Dashed { get; set; }
        /// <summary>
        /// The dash color
        /// </summary>
        /// <returns>Dash color</returns>
        public Color DashColor { get; set; } = Colors.Red;
        /// <summary>
        /// The dash background color
        /// </summary>
        /// <returns>Dash background color</returns>
        public Color DashBackgroundColor { get; set; } = Colors.Transparent;
        /// <summary>
        /// The pattern of dashes and gaps used to draw the box
        /// </summary>
        /// <returns>Dash pattern</returns>
        public float[] DashPattern { get; set; } = new float[] {6, 4};
        /// <summary>
        /// The width/height of the dashes when drawn 
        /// </summary>
        /// <returns>Dash size</returns>
        public float DashSize { get; set; }
        /// <summary>
        /// The distance within the dash pattern where dashes begin
        /// </summary>
        /// <returns>Dash size</returns>
        public int DashOffset { get; set; }
        /// <summary>
        /// Constructor
        /// </summary>
        public BoxDrawable() 
        { 
        }
        /// <summary>
        /// Draws the box on the canvas
        /// </summary>
        /// <param name="canvas">Target canvas</param>
        /// <param name="dirtyRect">The rectangular region into which the box is to be drawn</param>
        /// <returns>Nothing</returns>
        public void Draw(ICanvas canvas, RectF dirtyRect)
        {
            if(Dashed)
            {
                canvas.StrokeColor = DashColor;
                canvas.StrokeSize = DashSize;
                canvas.StrokeDashPattern = DashPattern;
                canvas.StrokeDashOffset = DashOffset;
            }
            canvas.FillColor = Dashed ? DashBackgroundColor : BackgroundColor;
            canvas.FillRectangle(0, 0, dirtyRect.Width, dirtyRect.Height);
            if (Dashed)
            {
                if (dirtyRect.Width > dirtyRect.Height)
                    canvas.DrawLine(0, dirtyRect.Height / 2, dirtyRect.Width, dirtyRect.Height / 2);
                else
                    canvas.DrawRectangle(dirtyRect.Width / 2 - DashSize / 2, 0, DashSize, dirtyRect.Height);
            }
        }
    }
    /// <summary>
    /// The `BorderBox` class represents a border that can be drawn either as a solid box or as an animated dashed line
    /// </summary>
    public class BorderBox : GraphicsView
    {
        // Private property data
        private Color color = Colors.Black;
        private bool dashedAnimationEnabled = false;
        private float dashThickness = 1.0F;
        private int animationCycleIndex = 0;

        private const int DASH_LENGTH = 6;
        private const int GAP_LENGTH = 4;
        private const int TOTAL_ANIMATION_PATTERN_LENGTH = DASH_LENGTH + GAP_LENGTH;
        private float[] dashPattern = new float[2] { DASH_LENGTH, GAP_LENGTH };

        /// <summary>
        /// Pan updated delegate handler
        /// </summary>
        /// <param name="sender">The border box sender</param>
        /// <param name="e">Pan updated event arguments</param>
        public delegate void PanUpdatedHandler(BorderBox sender, PanUpdatedEventArgs e);
        /// <summary>
        /// The registered pan updated event handler (if any)
        /// </summary>
        /// <returns>Pan updated event handler</returns>
        public event PanUpdatedHandler? PanUpdated;
        /// <summary>
        /// Constructor
        /// </summary>
        /// <param name="hoverIcon">The icon to be displayed when the mouse pointer hovers over the object</param>
        /// <param name="dashThickness">The thickness of the dashes. This is their height for a horizontal box and their width for a vertical box.</param>
        public BorderBox(Icon hoverIcon, float dashThickness)
        {
            HoverIcon = hoverIcon;
            DashThickness = dashThickness;
            RegisterEventHandlers();
            InitializeDrawable();
        }
        /// <summary>
        /// The icon to be displayed when the mouse pointer hovers over the object
        /// </summary>
        /// <returns>Hover icon</returns>
        public Icon HoverIcon { get; set; }
        /// <summary>
        /// The color of the box
        /// </summary>
        /// <returns>Box color</returns>
        public Color Color
        {
            get => color;
            set
            {
                if (color != value)
                {
                    color = value;
                    UpdateDrawable();
                }
            }
        }
        /// <summary>
        /// The index within the box animation cycle. Only applies when dash animation is enabled.
        /// </summary>
        /// <returns>Animation cycle index</returns>
        public int AnimationCycleIndex
        {
            get => animationCycleIndex;
            set
            {
                animationCycleIndex = value >= 0 && value < TOTAL_ANIMATION_PATTERN_LENGTH ? value : 0;
            }
        }
        /// <summary>
        /// The pattern of dashes and gaps used to draw the box. Only applies when dash animation is enabled.
        /// </summary>
        /// <returns>Dash pattern</returns>
        public float[] DashPattern { get => dashPattern; }
        /// <summary>
        /// The thickness of the dashes when the border is drawn. This is their height for a horizontal box and their width for a vertical box.
        /// Only applies when dash animation is enabled.
        /// </summary>
        /// <returns>Dash thickness</returns>
        public float DashThickness
        {
            get => dashThickness;
            set
            {
                if (dashThickness != value)
                {
                    dashThickness = value;
                    UpdateDrawable();
                }
            }
        }
        /// <summary>
        /// Initializes the drawable object
        /// </summary>
        private void InitializeDrawable() {
            Drawable = new BoxDrawable();
            UpdateDrawable();
        }
        /// <summary>
        /// Updates the drawable object
        /// </summary>
        private void UpdateDrawable()
        {
            if (Drawable as BoxDrawable is BoxDrawable drawable)
            {
                drawable.BackgroundColor = Color;
                drawable.Dashed = dashedAnimationEnabled;
                drawable.DashSize = DashThickness;
                drawable.DashPattern = DashPattern;
                drawable.DashColor = Color;
                drawable.DashBackgroundColor = Colors.White;
                drawable.DashOffset = AnimationCycleIndex;
                Invalidate();
            };
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
        /// Handles the pan updated event, triggering any registed PanUpdated handlers
        /// </summary>
        private void OnPanUpdated(PanUpdatedEventArgs e)
        {
            if (PanUpdated != null)
            {
                PanUpdated.Invoke(this, e);
            }
        }
        /// <summary>
        /// Enables or disables dash animation
        /// </summary>
        /// <param name="enable">Flag indicating whether to enable animation</param>
        public void EnableDashAnimation(bool enable)
        {
            if (enable != dashedAnimationEnabled)
            {
                dashedAnimationEnabled = enable;
                ResetAnimationCycleIndex();
                UpdateDrawable();
            }
        }
        /// <summary>
        /// Updates the dash animation displayed by one cycle. Only applies when dash animation is enabled.
        /// </summary>
        public void UpdateDashAnimation()
        {
            if (dashedAnimationEnabled)
            {
                IncrementAnimationCycleIndex();
                UpdateDrawable();
            }
        }
        /// <summary>
        /// Resets the dash animation cycle index to the starting index
        /// </summary>
        private void ResetAnimationCycleIndex()
        {
            AnimationCycleIndex = 0;
        }
        /// <summary>
        /// Increments the dash animation cycle index by one, resetting to zero if
        /// the end of the dash pattern is reached.
        /// </summary>
        private void IncrementAnimationCycleIndex()
        {
            AnimationCycleIndex = AnimationCycleIndex + 1;
        }
    }
}
