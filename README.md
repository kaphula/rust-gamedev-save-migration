# rust-gamedev-save-migration

Educational and experimental repository for exploring the problem of migrating game save files between different
versions of a game.

## Reasoning

Let us say you are creating a game which allows users to save their progress to their hard drives.
As you release the game and people start playing it they will progress on their own pace and create save files of their
progress.

After some time you as a developer decide to add new features or bug fixes to the game.
This means you will have to change the internal data structure and logic of the game compared to previous versions.

Once you apply those changes and release an update, the users' game will usually automatically update without their
consent.
Now, when they try to continue their game progress using a save file from previous version, your game needs to know how
to
migrate the
previous save file to the current version of the game. 

Nobody likes to be told that a silent update just dropped
and
now your previous progress is lost because attempting to use a previous save file will crash the game or simply does not work
otherwise, and therefore the only option to keep playing the game is to start a
new game (until next silent update drops).

How do you prepare and manage the complexity of converting old save files to be compatible with your game's latest
version?

## Run

`cargo run`