using Maze.Maui.App.Controls.Pointer;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    internal class BoxDrawable : IDrawable
    {
        public Color BackgroundColor { get; set; } = Colors.Transparent;

        public bool Dashed { get; set; }
        public Color DashColor { get; set; } = Colors.Red;

        public Color DashBackgroundColor { get; set; } = Colors.Transparent;

        public float[] DashPattern { get; set; } = new float[] {6, 4};

        public float DashSize { get; set; }

        public int DashOffset { get; set; }

        public BoxDrawable() 
        { 
        }

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

    public class BorderBox : GraphicsView
    {
        public delegate void PanUpdatedHandler(BorderBox sender, PanUpdatedEventArgs e);
        public event PanUpdatedHandler? PanUpdated;

        private Icon HoverIcon { get; }
        private Color color = Colors.Black;
        private bool dashedAnimationEnabled = false;
        private float dashThickness = 1.0F;
        private int animationCycle = 0;

        private const int DASH_LENGTH = 6;
        private const int GAP_LENGTH = 4;
        private const int TOTAL_ANIMATION_PATTERN_LENGTH = DASH_LENGTH + GAP_LENGTH;

        private float[] dashPattern = new float[2] { DASH_LENGTH, GAP_LENGTH};

        public BorderBox(Icon hoverIcon, float dashThickness)
        {
            HoverIcon = hoverIcon;
            DashThickness = dashThickness;
            RegisterEventHandlers();
            InitializeDrawable();
        }


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

        public int AnimationCycle
        {
            get => animationCycle;
            set
            {
                animationCycle = value >= 0 && value < TOTAL_ANIMATION_PATTERN_LENGTH ? value : 0;
            }
        }

        public float[] DashPattern { get => dashPattern; } 

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

        private void InitializeDrawable() {
            Drawable = new BoxDrawable();
            UpdateDrawable();
        }

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
                drawable.DashOffset = AnimationCycle;
                Invalidate();
            };
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

        public void EnableDashAnimation(bool enable)
        {
            if (enable != dashedAnimationEnabled)
            {
                dashedAnimationEnabled = enable;
                ResetAnimationCycle();
                UpdateDrawable();
            }
        }
        public void UpdateDashAnimation()
        {
            if (dashedAnimationEnabled)
            {
                IncrementAnimationCycle();
                UpdateDrawable();
            }
        }

        private void ResetAnimationCycle()
        {
            AnimationCycle = 0;
        }

        private void IncrementAnimationCycle()
        {
            AnimationCycle = AnimationCycle + 1;
        }
    }
}
