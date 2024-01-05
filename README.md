This a simple terminal program that can be used to keep track of shifts, and calculate your salery from them.
All the shifts are stored in a simple sql database, and the salery is calculated from a set of user-defined "rules" located in the "wage_bonuses_map.json"-file.

There are six commands:
- Add - add a shift
- remove - remove a shift
- list - list the shifts
- calculate - calculate salery from the shifts
- edit-shift - edit a shift
- drop-database - deletes the database and all shifts

Of course there is also --help or -h that give a better description of what the six commands do.

Most commands have flags and arguments that can be used to specify behavior.
Eg. "list" can list all shifts or just the shifts in this month.
-h can also be used to get a description of these.

The project is structured in a fairly simple way: main() determines what command is given and calls the corresponding functions located in other files, to keep things somewhat tidy.
This project is a mess, but it works. If anybody else wants to use it, feel free to.
