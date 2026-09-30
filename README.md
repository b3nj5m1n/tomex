<div align="center">
  <h1>tomex</h1>
  <h4>Track reading habits & library with a strict datamodel</h4>
</div>

<p align="center">
  <img src="https://github.com/user-attachments/assets/e1d6e393-d7a6-460a-9ddd-cf268f792a2e" width="500" alt="Demonstration of tomex cli tool" />
</p>

This project was born out of a frustration with existing solutions' inconsistent datamodels, especially the inability to separate between different editions of the same book in most contexts.
In tomex's datamodel, there are books and editions of books. You can read an edition, but not a book by itself.
After you've read a book, you can review the book (how much you liked it, whether you perceived the pacing as fast or slow, found the mood to be dark, funny, mysterious, etc.) and separately the edition (quality of a translation or an expensive special edition).

## State of the project

This is very much unfinished, and I haven't had the time or motivation to work on it in a few years.
Most of the features I initially envisioned are incomplete or missing altogether.
At the moment, this is basically a glorified CRUD app.
I was also still pretty new to rust when I wrote this.

That being said, I've been using this without any problems for three or four years now to keep track of my book related data (and then exporting that data to storygraph periodically when I want to see stats).

## Installing

First clone the repository
```bash
git clone https://github.com/b3nj5m1n/tomex
cd tomex
```
You can then either build directly with cargo:
```bash
cargo build --release
```
Or use the provided nix flake:
```bash
nix build .#tomex
```

Using nix, you can also run `tomex` directly:
```bash
nix run github:b3nj5m1n/tomex -- [arguments]
```

## Usage

Run `tomex help` to get an overview of available subcommands.
You can also add `--help` to any subcommand to get more info about those.

### Database location

By default, `tomex` stores the database in `~/.local/share/tomex/database`. You can change this using the environment variable `TOMEX_DATABASE_LOCATION`, for example:
```bash
TOMEX_DATABASE_LOCATION="./database" tomex
```

### Adding books

Running `tomex add book` will launch an interactive prompt for adding a book. You'll then want to run `tomex add edition` to add an edition to that book.

You can also pre-fill a lot of the data using `tomex add by_isbn`,
this will query [OpenLibrary](https://openlibrary.org/). Note that you currently can't supply the ISBN as an argument to this command, it will instead launch a prompt.

<p align="center">
  <img src="https://github.com/user-attachments/assets/d9ad744c-d52f-45d6-8d02-d5b7556ca8a4" width="700" alt="Demonstration of adding a book by ISBN" />
</p>

Finally, for adding a lot of books at once, you can use `tomex listen`. This will start a webserver, currently hardcoded to `loaclhost` on port `3000`.
You can then use a barcode scanner on your phone capable of sending scanning results to this webserver. It's expecting ISBNs to be send to the `api/isbn?content=` endpoint, with the ISBN added at the end.

For example, to use this with [BinaryEye](https://github.com/markusfisch/BinaryEye), enable "Forward scans" in BinaryEye's settings and set "URL to forward to" to the address tomex is listening on, for example `http://192.168.178.1:3000/api/isbn?content=`. Obviously, make sure you're on the same network and that port `3000` is open.

After the webserver receives an ISBN, it'll try to query OpenLibrary and launch an interactive prompt to add the book. After you've finished adding the book (or it fails because OpenLibrary doesn't know it yet, make sure to help out and add it), it'll start listening for further ISBNs.

### Tracking reading progress

Use `tomex add progress` and select the edition you're reading, you can then choose whether you want to input the time using an interactive datepicker, or by inputting a unix timestamp.
Now, choose whether at this time you started the book, finished it, or provide a progress update with the page you're currently on.

### Managing other data

The `tomex add` subcommand can add a lot more than books and editions. While adding a book or edition, you can add a new author in the same dialog flow, but this isn't currently possible with most other things. In particular, if you want to select other genres or publishers, you have to create them manually using the respective commands `tomex add genre`, `tomex add publisher` first.
See `tomex add --help` for more info.

You can similarly retrieve this information using the `tomex query` subcommand, for example, `tomex query edition` will list the editions in your database. Most notably, `tomex query progress` will display a chronological list of your progress updates.

### Other commands

`tomex backup` creates a full backup of your database as json, which can be restored to a sqlite database using `tomex restore`.

`tomex export` exports your data in a format that allows you to import it in goodreads/storygraph/bookwyrm. Some information might be lost during this conversion. Be very careful about importing this into your existing accounts.

`tomex remove` can delete entries in your database.

You can also start a repl using `tomex repl`.




