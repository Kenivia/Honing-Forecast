using LostArk.Archive.Core.Archives;

// Decrypts the tables the scripts read out of the game's archives, and its fonts:
// extract <game folder> <tables folder> <fonts folder>
var game = args[0];
var output = args[1];
var fonts = args[2];
string[] tables = ["Item", "GameMsg", "RandomBoxBase", "RandomBoxEntity", "DropBase", "DropEntity", "Money", "StagePerReward", "PaidActionCost", "ZoneEpicGate", "ContentsRequirement", "ItemEnhanceCommon", "ItemQualityOption", "ItemEnhanceMaterial", "TrinityInferno", "TrinityInfernoReward", "TrinityDivine", "TrinityDivineEnergyPhase", "PCRoomDailyReward", "GuideWeeklyMissionReward", "DirectShop", "DirectShopMeta", "AcquireLimit", "ContentsSeason"];
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

// every font and font map, by its path under Binaries/Fonts (languages share file names)
var fontArchive = Path.Combine(game, "EFGame", "font.lpk");
var fontResult = ArchiveInspector.Inspect(fontArchive);
foreach (var entry in fontResult.Entries)
{
    var path = entry.FullPath.Replace('\\', '/');
    var at = path.IndexOf("Binaries/Fonts/");
    if (at < 0) continue;
    var target = Path.Combine(fonts, path[(at + "Binaries/Fonts/".Length)..]);
    ArchiveExtractor.TryExtractEntry(fontArchive, fontResult.Format, entry, out var data, out _, out _, out _);
    Directory.CreateDirectory(Path.GetDirectoryName(target)!);
    File.WriteAllBytes(target, data);
    Console.WriteLine($"{path[(at + "Binaries/Fonts/".Length)..]} ({data.Length:N0} bytes)");
}
