# stat tracking bot

A Discord Bot to track statistics by measuring how many images have been sent in a specific channel.

It constructs a graph and can send that graph in a channel if you would like it to.

## Set-up Guide

Create a Discord Bot and give it the `Message Content` Intent

Then, in the project directory, create a `.env` file and write `DISCORD_TOKEN=` followed by the bot's token.

Now create a `tracking.json` file with the following layout

```json
{
    "GUILD_ID" : {
        "to_track" : [
            "CHANNEL_NAME",
            "CHANNEL_NAME",
            "CHANNEL_NAME"
        ],
        "output_channel" : "CHANNEL_NAME"
    },

    "GUILD_ID" : ...,

    "GUILD_ID" : ...
}
```

You can then use `cargo run` to run track your statistics!

The graphs will be saved in an `out` folder
and the bot will ask you whether or not you want to send them into the output channel.
