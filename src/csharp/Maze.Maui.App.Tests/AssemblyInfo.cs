using Xunit;

// Tests register recipients on `WeakReferenceMessenger.Default`, a
// process-global singleton. Running tests in parallel lets one test's
// `Send(...)` fan out to ViewModel instances built by other concurrent
// tests, mutating their ObservableCollection mid-enumeration. The whole
// suite runs in ~1s, so serialising it costs nothing.
[assembly: CollectionBehavior(DisableTestParallelization = true)]
