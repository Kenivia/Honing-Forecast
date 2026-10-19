using LostArk.Archive.Core.Archives;

// Decrypts the tables the scripts read out of the game's archives: extract <game folder> <output folder>
var game = args[0];
var output = args[1];
string[] tables = ["Item", "GameMsg", "RandomBoxBase", "RandomBoxEntity", "DropBase", "DropEntity", "Money"];
var wanted = tables.Select(table => $"TableData/EFTable_{table}.db").Append("XmlData/IconInfo.loa").ToArray();

Directory.CreateDirectory(output);
foreach (var name in new[] { "data2.lpk", "data3.lpk" })
{
    var archive = Path.Combine(game, "EFGame", name);
    var result = ArchiveInspector.Inspect(archive);
    foreach (var entry in result.Entries.Where(entry => wanted.Any(entry.FullPath.Replace('\\', '/').EndsWith)))
    {
        ArchiveExtractor.TryExtractEntry(archive, result.Format, entry, out var data, out _, out _, out _);
        File.WriteAllBytes(Path.Combine(output, Path.GetFileName(entry.FullPath)), data);
        Console.WriteLine($"{Path.GetFileName(entry.FullPath)} ({data.Length:N0} bytes)");
    }
}
