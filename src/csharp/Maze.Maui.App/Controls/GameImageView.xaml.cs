namespace Maze.Maui.App
{
    /// <summary>
    /// A rounded-rectangle game thumbnail: a fixed-size framed square showing the
    /// image decoded from <see cref="Bytes"/>, falling back to the generic 3D icon
    /// when there are none. Purely presentational — the bearer-authenticated fetch
    /// lives in the view models / <see cref="Services.IGameLibraryService"/>, which
    /// hand this control the raw bytes (so the view-model layer stays free of MAUI
    /// image types); this control owns the <see cref="ImageSource"/> conversion.
    /// Mirrors <see cref="AvatarView"/>, differing only in the rounded-rectangle
    /// (vs circular) frame and the placeholder.
    /// </summary>
    public partial class GameImageView : ContentView
    {
        /// <summary>The game image PNG bytes to show, or <c>null</c>/empty for the placeholder.</summary>
        public static readonly BindableProperty BytesProperty = BindableProperty.Create(
            nameof(Bytes), typeof(byte[]), typeof(GameImageView), default(byte[]),
            propertyChanged: OnBytesChanged);

        /// <summary>The rendered edge length, in device-independent units.</summary>
        public static readonly BindableProperty SizeProperty = BindableProperty.Create(
            nameof(Size), typeof(double), typeof(GameImageView), 64.0,
            propertyChanged: OnSizeChanged);

        /// <summary>The image file shown when there are no <see cref="Bytes"/> — the kind's placeholder art (default the generic 3D-game icon).</summary>
        public static readonly BindableProperty PlaceholderFileProperty = BindableProperty.Create(
            nameof(PlaceholderFile), typeof(string), typeof(GameImageView), "play3d.png",
            propertyChanged: OnPlaceholderChanged);

        public GameImageView()
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

        public string PlaceholderFile
        {
            get => (string)GetValue(PlaceholderFileProperty);
            set => SetValue(PlaceholderFileProperty, value);
        }

        private static void OnBytesChanged(BindableObject bindable, object oldValue, object newValue)
            => ((GameImageView)bindable).ApplyBytes(newValue as byte[]);

        private static void OnSizeChanged(BindableObject bindable, object oldValue, object newValue)
            => ((GameImageView)bindable).ApplySize((double)newValue);

        private static void OnPlaceholderChanged(BindableObject bindable, object oldValue, object newValue)
        {
            var view = (GameImageView)bindable;
            // Only affects what's shown while there are no bytes.
            if (view.Bytes is not { Length: > 0 })
                view.ApplyBytes(null);
        }

        private bool _hasImage;

        private void ApplyBytes(byte[]? bytes)
        {
            _hasImage = bytes is { Length: > 0 };
            // Re-invokable stream factory (MAUI may read the source more than
            // once), each call a fresh MemoryStream over the captured bytes.
            GameImage.Source = bytes is { Length: > 0 }
                ? ImageSource.FromStream(() => new MemoryStream(bytes))
                : ImageSource.FromFile(string.IsNullOrEmpty(PlaceholderFile) ? "play3d.png" : PlaceholderFile);
            ApplyPadding();
        }

        private void ApplySize(double size)
        {
            ImageFrame.WidthRequest = size;
            ImageFrame.HeightRequest = size;
            ApplyPadding();
        }

        // Inset a placeholder glyph so it sits clear of the frame (mirroring the
        // web client's thumbnail padding); a real uploaded image fills the frame
        // edge-to-edge. Proportional to Size so it holds at any thumbnail size.
        private void ApplyPadding()
            => ImageFrame.Padding = _hasImage ? 0 : new Thickness(Size * 0.1);
    }
}
