using Microsoft.Maui.Graphics;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Maze.Maui.App.Controls.InteractiveGrid
{
    public class FrameBorder
    {
        SelectionFrame parentFrame;

        private readonly BoxView box;
        private readonly BoxView grip;

        static private Color DEFAULT_COLOR = Colors.Black;
        const double DEFAULT_WIDTH = 2.0;
        const double DEFAULT_GRIP_SIZE = 4.0;

        Color color = DEFAULT_COLOR;
        double gripSize = DEFAULT_GRIP_SIZE;

        public enum Location
        {
            Top = 0,
            Left = 1,
            Bottom = 2,
            Right = 3
        }

        public SelectionFrame ParentFrame { get => parentFrame; }

        public Grid ParentGrid { get => ParentFrame.ParentGrid; }

        public Color Color
        {
            get => color;
            set
            {
                color = value;
                UpdateColor();
            }
        }

        public double GripSize
        {
            get => gripSize;
            set
            {
                gripSize = value;
                UpdateGripSize();
            }
        }

        public FrameBorder(SelectionFrame parentFrame)
        {
            this.parentFrame = parentFrame;
            box = NewBox();
            grip = NewGrip();
        }

        private BoxView NewBox()
        {
            return new BoxView
            {
                Color = Color,
                IsVisible = false,
                HeightRequest = 0,
                WidthRequest = 0,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = true
            };
        }

        private BoxView NewGrip()
        {
            return new BoxView
            {
                Color = Color,
                IsVisible = false,
                HeightRequest = GripSize,
                WidthRequest = GripSize,
                CornerRadius = GripSize / 2.0,
                HorizontalOptions = LayoutOptions.Start,
                VerticalOptions = LayoutOptions.Start,
                InputTransparent = true
            };
        }

        private void UpdateColor()
        {
            box.Color = Color;
            grip.Color = Color;
        }

        private void UpdateGripSize()
        {
            grip.HeightRequest = GripSize;
            grip.WidthRequest = GripSize;
            grip.CornerRadius = GripSize / 2.0;
        }

        public void Show(bool show)
        {
            box.IsVisible = show;
            grip.IsVisible = show;
        }

        public void AddToGrid()
        {
            AddViewToGrid(box);
            AddViewToGrid(grip);
        }

        private void AddViewToGrid(BoxView view)
        {
            ParentGrid.SetRow(view, 0);
            ParentGrid.SetColumn(view, 0);
            ParentGrid.Children.Add(view);
        }

        public void SetPosition(Location location, int row, int column, double width, double height, Thickness margin)
        {
            ParentGrid.SetRow(box, row);
            ParentGrid.SetColumn(box, column);
            box.WidthRequest = width;
            box.HeightRequest = height;
            box.Margin = margin;

            ParentGrid.SetRow(grip, row);
            ParentGrid.SetColumn(grip, column);
            grip.TranslationX = GetGripTranslationX(location, width);
            grip.TranslationY = GetGripTranslationY(location, height);
            grip.Margin = margin;
        }

        private double GetGripTranslationX(Location location, double width)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (location)
            {
                case Location.Top:
                    return -GripSize / 2.0;
                case Location.Left:
                    return -GripSize / 2.0;
                case Location.Bottom:
                    return width - GripSize / 2.0; ;
                case Location.Right:
                    return 1.0 - GripSize / 2.0;
            }
            return 0.0;
        }

        private double GetGripTranslationY(Location location, double height)
        {
            bool singleRow = ParentFrame.RowCount == 1;
            bool singleColumn = ParentFrame.ColumnCount == 1;

            switch (location)
            {
                case Location.Top:
                    return -GripSize / 2.0;
                case Location.Left:
                    return height - GripSize / 2.0;
                case Location.Bottom:
                    return height - GripSize / 2.0;
                case Location.Right:
                    return -GripSize / 2.0;
            }
            return 0.0;
        }
    }
}