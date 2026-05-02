// Stub of MAUI's Microsoft.Maui.Controls.QueryPropertyAttribute. The real
// attribute is consumed by Shell at runtime to populate properties from
// route parameters; tests bypass Shell entirely and set properties
// directly, so a no-op stub is sufficient to satisfy the compiler when
// ChangePasswordViewModel.cs is file-linked into this non-MAUI project.
namespace Microsoft.Maui.Controls
{
    [AttributeUsage(AttributeTargets.Class, AllowMultiple = true)]
    internal sealed class QueryPropertyAttribute : Attribute
    {
        public QueryPropertyAttribute(string name, string queryId)
        {
            Name = name;
            QueryId = queryId;
        }

        public string Name { get; }
        public string QueryId { get; }
    }
}
