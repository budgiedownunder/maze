using Windows.Devices.Input;

namespace Maze.Maui.App.Services
{
    public class DeviceTypeService : IDeviceTypeService
    {
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