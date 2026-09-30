# When in a nix environment, run with bashInteractive
# to get the propper prompt: nix-shell -p bashInteractive

Output demo.gif

Set FontSize 30
Set Width 1920
Set Height 1080

Env TEMP_DIR "/tmp/de80733d-24e4-4f6e-a99f-47905b83c83f"
Env DB_NAME "b0ebf4f4-8fe3-49e6-a89d-a1b603c87011"
Env TOMEX_DATABASE_LOCATION "./tmp/de80733d-24e4-4f6e-a99f-47905b83c83f/b0ebf4f4-8fe3-49e6-a89d-a1b603c87011"

Hide

Type "mkdir -p $TEMP_DIR && rm -f $TEMP_DIR/$DB_NAME"
Enter
Type "TOMEX_DATABASE_LOCATION=$TEMP_DIR/$DB_NAME nix run github:b3nj5m1n/tomex#tomex -- restore ./dummy_data.json"
Enter
Sleep 1s
Type "clear && TOMEX_DATABASE_LOCATION=$TEMP_DIR/$DB_NAME nix run github:b3nj5m1n/tomex#tomex -- repl"
Enter

Show

Sleep 3

Type "query book"
Sleep 1s
Enter

Sleep 5s

Type "query progress"
Sleep 1s
Enter

Sleep 5s

Type "query edition"
Sleep 1s
Enter

Sleep 5s
