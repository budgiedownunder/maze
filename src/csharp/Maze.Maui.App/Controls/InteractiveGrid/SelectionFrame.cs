namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class SelectionFrame
    {
        private readonly Grid parent;
        private readonly BoxView outerBox;
        //private List<BoxView>? selectionFrameGrips;


        static private Color DEFAULT_BORDER_COLOR = Colors.Black;

        private Color borderColor = DEFAULT_BORDER_COLOR;

        public Color BorderColor
        {
            get => borderColor;
            set
            {
                borderColor = value;
                outerBox.Color = borderColor;
            }
        }

        public SelectionFrame(Grid parent)
        {
            this.parent = parent;
            outerBox = new BoxView
            {
                Color = BorderColor,
                IsVisible = false,
                HeightRequest = 1.0,
                WidthRequest = 1.0,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = true
            };
        }

        public void Show(bool show)
        {
            outerBox.IsVisible = show;
        }

        public void AddToGrid()
        {
            parent.SetRow(outerBox, 0);
            parent.SetColumn(outerBox, 0);

            parent.Children.Add(outerBox);
        }

        public void SetLocation(int row, int column, double width, double height)
        {
            parent.SetRow(outerBox, row);
            parent.SetColumn(outerBox, column);
            outerBox.WidthRequest = width;
            outerBox.HeightRequest = height;
        }
    }
}


// Initialize grip points (one for each side: top, bottom, left, right)
/*      selectionFrameGrips = new List<BoxView>();
      for (int i = 0; i < 4; i++)
      {
          var grip = new BoxView
          {
              Color = Colors.Red,
              WidthRequest = 20,
              HeightRequest = 20,
              CornerRadius = 10,
              IsVisible = false
          };
          selectionFrameGrips.Add(grip);
          this.Children.Add(grip); 
      }
*/
