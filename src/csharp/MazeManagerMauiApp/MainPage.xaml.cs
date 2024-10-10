namespace MazeMauiApp
{
    using System;
    using Maze.Api;

    public partial class MainPage : ContentPage
    {
        int count = 0;

        public MainPage()
        {
            InitializeComponent();
        }


        private void OnCounterClicked(object sender, EventArgs e)
        {
            count += 1;
            using (Maze maze = new Maze(10, 20))
            {
                if (count == 1)
                    CounterBtn.Text = $"Clicked {count} time (maze size = {maze.RowCount} rows x {maze.ColCount} columns";
                else
                    CounterBtn.Text = $"Clicked {count} times (maze size = {maze.RowCount} rows x {maze.ColCount} columns";

                SemanticScreenReader.Announce(CounterBtn.Text);
            }
        }
    }
}
