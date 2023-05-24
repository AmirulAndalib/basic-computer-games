using System.Diagnostics.CodeAnalysis;
using static King.Resources.Resource;

namespace King;

internal static class IOExtensions
{
    internal static bool TryReadGameData(this IReadWrite io, IRandom random, [NotNullWhen(true)] out Reign? reign)
    {
        if (io.TryReadValue(SavedYearsPrompt, v => v < Reign.MaxTerm, SavedYearsError(Reign.MaxTerm), out var years) &&
            io.TryReadValue(SavedTreasuryPrompt, out var rallods) &&
            io.TryReadValue(SavedCountrymenPrompt, out var countrymen) &&
            io.TryReadValue(SavedWorkersPrompt, out var workers) &&
            io.TryReadValue(SavedLandPrompt, v => v is > 1000 and <= 2000, SavedLandError, out var land))
        {
            reign = new Reign(io, random, new Country(io, random, rallods, countrymen, workers, land), years + 1);
            return true;
        }

        reign = default;
        return false;
    }

    internal static bool TryReadValue(this IReadWrite io, string prompt, out float value, params ValidityTest[] tests)
    {
        while (true)
        {
            var response = value = io.ReadNumber(prompt);
            if (response == 0) { return false; }
            if (tests.All(test => test.IsValid(response, io))) { return true; }
        } 
    }

    internal static bool TryReadValue(this IReadWrite io, string prompt, out float value)
        => io.TryReadValue(prompt, _ => true, "", out value);
    
    internal static bool TryReadValue(
        this IReadWrite io,
        string prompt,
        Predicate<float> isValid,
        string error,
        out float value)
        => io.TryReadValue(prompt, isValid, () => error, out value);

    internal static bool TryReadValue(
        this IReadWrite io,
        string prompt,
        Predicate<float> isValid,
        Func<string> getError,
        out float value)
    {
        while (true)
        {
            value = io.ReadNumber(prompt);
            if (value < 0) { return false; }
            if (isValid(value)) { return true; }
            
            io.Write(getError());
        }
    }
}
