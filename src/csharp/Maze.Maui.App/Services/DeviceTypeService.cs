namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Defines functionality for the current device type
    /// </summary>
    public interface IDeviceTypeService
    {
        /// <summary>
        /// Returns whether the current device is a touch-only device
        /// </summary>
        /// <returns>Boolean</returns>
        bool IsTouchOnlyDevice();
    }
}