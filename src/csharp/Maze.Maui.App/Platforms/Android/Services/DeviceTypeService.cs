namespace Maze.Maui.App.Services
{
    /// <summary>
    /// Defines functionality for the current device type
    /// </summary>
    public class DeviceTypeService : IDeviceTypeService
    {
        /// <summary>
        /// Returns whether the current device is a touch-only device
        /// </summary>
        /// <returns>Boolean</returns>
        public bool IsTouchOnlyDevice() => true;
    }
}