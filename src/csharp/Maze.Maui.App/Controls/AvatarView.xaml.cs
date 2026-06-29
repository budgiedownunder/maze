namespace Maze.Maui.App
{
    /// <summary>
    /// A circular user avatar: a fixed-size round frame (thin black ring,
    /// matching the web client) showing the image decoded from <see cref="Bytes"/>,
    /// falling back to the generic placeholder when there are none. Purely
    /// presentational — the bearer-authenticated fetch lives in the view models /
    /// <see cref="Services.IAvatarService"/>, which hand this control the raw
    /// bytes (so the view-model layer stays free of MAUI image types); this
    /// control owns the <see cref="ImageSource"/> conversion.
    /// </summary>
    public partial class AvatarView : ContentView
    {
        // The shipped generic placeholder (byte-identical to the web client's),
        // shown whenever the user has no avatar.
        private static readonly ImageSource Placeholder = ImageSource.FromFile("avatar_placeholder.png");

        /// <summary>The avatar PNG bytes to show, or <c>null</c>/empty for the placeholder.</summary>
        public static readonly BindableProperty BytesProperty = BindableProperty.Create(
            nameof(Bytes), typeof(byte[]), typeof(AvatarView), default(byte[]),
            propertyChanged: OnBytesChanged);

        /// <summary>The rendered diameter, in device-independent units.</summary>
        public static readonly BindableProperty SizeProperty = BindableProperty.Create(
            nameof(Size), typeof(double), typeof(AvatarView), 32.0,
            propertyChanged: OnSizeChanged);

        public AvatarView()
        {
            InitializeComponent();
            ApplySize(Size);
            ApplyBytes(Bytes);
        }

        public byte[]? Bytes
        {
            get => (byte[]?)GetValue(BytesProperty);
            set => SetValue(BytesProperty, value);
        }

        public double Size
        {
            get => (double)GetValue(SizeProperty);
            set => SetValue(SizeProperty, value);
        }

        private static void OnBytesChanged(BindableObject bindable, object oldValue, object newValue)
            => ((AvatarView)bindable).ApplyBytes(newValue as byte[]);

        private static void OnSizeChanged(BindableObject bindable, object oldValue, object newValue)
            => ((AvatarView)bindable).ApplySize((double)newValue);

        private void ApplyBytes(byte[]? bytes)
        {
            // Re-invokable stream factory (MAUI may read the source more than
            // once), each call a fresh MemoryStream over the captured bytes.
            AvatarImage.Source = bytes is { Length: > 0 }
                ? ImageSource.FromStream(() => new MemoryStream(bytes))
                : Placeholder;
        }

        private void ApplySize(double size)
        {
            Ring.WidthRequest = size;
            Ring.HeightRequest = size;
        }
    }
}
