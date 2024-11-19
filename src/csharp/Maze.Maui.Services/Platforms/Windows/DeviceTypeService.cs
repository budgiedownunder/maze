using Windows.Devices.Input;

namespace Maze.Maui.Services
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
        public bool IsTouchOnlyDevice()
        {
            var touchCapabilities = new TouchCapabilities();
            var mouseCapabilities = new MouseCapabilities();
            var keyboardCapabilities = new KeyboardCapabilities();

            bool hasTouch = touchCapabilities.TouchPresent > 0;
            bool hasMouse = mouseCapabilities.MousePresent > 0;
            bool hasKeyboard = keyboardCapabilities.KeyboardPresent > 0;

            return hasTouch && !hasMouse && !hasKeyboard;
        }
    }
}