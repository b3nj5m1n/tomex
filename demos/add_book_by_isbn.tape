# When in a nix environment, run with bashInteractive
# to get the propper prompt: nix-shell -p bashInteractive
# This won't select the correct publication date
# when not run on the 30. of september 2026

Output add_book_by_isbn.gif

Set FontSize 30
Set Width 1920
Set Height 1080

Env TEMP_DIR "/tmp/571139b1-b5ba-45b3-b930-c6a77feb54d4"
Env DB_NAME "9c2de0dc-6630-4ecd-ba7f-0e3a382aa36d"

Hide

Type "mkdir -p $TEMP_DIR && cd $TEMP_DIR && rm -f $DB_NAME"
Enter

Type "clear && TOMEX_DATABASE_LOCATION=./$DB_NAME nix run github:b3nj5m1n/tomex#tomex -- a isbn"
Enter
Sleep 2s

Show

# Prompt for isbn
Type "978-0-14-143957-0"
Enter

Sleep 3s

# Prompt for authors name, prefilled
Enter

Sleep 1s

# Prompt for authors date of birth, prefilled
Enter

Sleep 1s

# Prompt for authors date of death, prefilled
Enter

Sleep 1s

# Prompt for saving author
Enter

Sleep 1s

# Prompt for saving title
Enter

Sleep 1s

# Prompt for books release date
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Ctrl+Up
Sleep 700ms
Ctrl+Left
Sleep 500ms
Ctrl+Left
Sleep 500ms
Ctrl+Left
Sleep 500ms
Up@500ms 2
Right@500ms 4
Enter

Sleep 1s

# Genre prompt
Type@700ms ho
Sleep 500ms
Space
Sleep 1s
Enter
Sleep 1s

# Select series prompt
Escape
Sleep 1s

# Edition title prompt
Escape
Sleep 1s

# Edition description prompt
Escape
Sleep 1s

# Isbn prompt
Enter
Sleep 1s

# Single volume/multi-part prompt
Enter
Sleep 1s

# Language prompt
Space
Sleep 500ms
Enter
Sleep 1s

# Publisher prompt
Space
Sleep 500ms
Enter
Sleep 1s

# Edition format prompt
Enter
Sleep 1s
Enter # well apparently i fucked it up

Sleep 2s

