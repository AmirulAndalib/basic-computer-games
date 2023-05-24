using System.Reflection;
using System.Runtime.CompilerServices;

namespace Poetry.Resources;

internal static class Resource
{
    internal static class Streams
    {
        public static Stream Title => GetStream();
    }

    private static Stream GetStream([CallerMemberName] string? name = null) =>
        Assembly.GetExecutingAssembly().GetManifestResourceStream($"{typeof(Resource).Namespace}.{name}.txt")
            ?? throw new Exception($"Could not find embedded resource stream '{name}'.");
}