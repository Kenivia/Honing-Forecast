"""
Pulls what the scripts here read out of the installed game: the tables into scripts/game_files/tables
and the icon sheets into scripts/game_files/atlas. Neither is tracked. Run again after a game patch.

    python scripts/game_files/extract.py [game folder]

Needs the .NET SDK, lostark-explorer under game_file_reader/, and the Lost Ark build of umodel with
SDL2_64.dll in game_file_reader/umodel/.
"""

import shutil
import subprocess
import sys

from game import ATLAS, HERE, ROOT, TABLES

GAME = r"C:\Program Files (x86)\Steam\steamapps\common\Lost Ark"
UMODEL = ROOT / "game_file_reader" / "umodel" / "umodel_lostark_v7.exe"


def main():
    game = sys.argv[1] if len(sys.argv) > 1 else GAME
    subprocess.run(["dotnet", "run", "-c", "Release", "--project", HERE / "extract", "--", game, TABLES], check=True)
    shutil.rmtree(ATLAS, ignore_errors=True)
    packages = f"{game}/EFGame/ReleasePC/Packages"
    command = [UMODEL, "-export", "-png", "-na", "-game=lostark", "-nameresolve", f"-path={packages}", f"-out={ATLAS}", "EFUI_IconAtlas_*"]
    log = subprocess.run(command, check=True, capture_output=True, text=True).stdout
    print(log.strip().splitlines()[-1])


if __name__ == "__main__":
    main()
