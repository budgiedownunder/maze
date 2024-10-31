namespace Maze.Maui.App.Views
{
    using System;
    using Maze.Api;
    using Microsoft.Maui.Controls;
    
    public partial class MainPage : ContentPage
    {
        const String APP_TITLE = "MAZE";
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

        private void OnResetBtn_Clicked(object sender, EventArgs e)
        {
            DisplayAlert(APP_TITLE, "Reset", "OK");
        }

    }
}
