use core::panic;
use std::{process::exit, collections::HashMap, path::PathBuf, io::{self, Write}};

use chrono::{NaiveDateTime, Local, Weekday, Utc};
use lazy_static::lazy_static;
use question::{Question, Answer};
use serde::Deserialize;
use serenity::{prelude::{EventHandler, Context}, async_trait, model::prelude::{Ready, GuildChannel}};

use crate::plot::plot;
pub(crate) struct Handler;

const MESSAGE_BATCH: u64 = 50;

#[derive(Deserialize)]
struct GuildTracking {
    to_track: Vec<String>,
    output_channel: String,
}

lazy_static! {
    static ref TRACKING: HashMap<u64, GuildTracking> = {
        let tracking_json = include_str!("../tracking.json");
        serde_json::from_str(tracking_json).expect("Invalid `tracking.json` file")
    };
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for (guild_id, GuildTracking { to_track, output_channel }) in TRACKING.iter() {
            let guild = ctx.http.get_guild(*guild_id).await.unwrap_or_else(|err| {
                panic!("Couldn't get guild: {}", err);
            });

            println!("{} : {}", guild.name, guild.id);

            let channels = 
                ctx
                .http
                .get_channels(guild.id.0)
                .await
                .unwrap_or_else(|err| {
                    panic!("Couldn't get channels: {}", err);
                });

            let output_channel = channels
                .iter()
                .filter(|channel| &channel.name == output_channel)
                .nth(0)
                .or_else(|| {
                    println!("Couldn't find #{} in {}", output_channel, guild.name);
                    None
                });

            let tracking_channels: Vec<&GuildChannel> = 
                channels
                .iter()
                .filter(|channel| to_track.contains(&channel.name))
                .collect();

            for item_to_track in to_track {
                if !tracking_channels
                    .iter()
                    .any(|channel| &channel.name == item_to_track) {
                    println!("Couldn't find #{} in {}", item_to_track, guild.name)
                }
            }
            
            let mut saved = HashMap::new();

            for channel in tracking_channels {
                if !channel.is_text_based() { 
                    println!("#{} is not text based", channel.name)
                }

                let print_wheel = |flush: bool, n: usize| {
                    const WHEEL_PARTS: &[u8] = "|/-\\".as_bytes();

                    print!(
                        " {} ",
                        WHEEL_PARTS[n % WHEEL_PARTS.len()] as char
                    );

                    if flush {
                        io::stdout().flush().unwrap();
                    }
                };

                let print_tracking = |flush: bool| {
                    print!("\x1B[2K\r");
                    print!("  tracking #{}", channel.name);

                    if flush {
                        io::stdout().flush().unwrap();
                    }
                };

                let mut timestamps = Vec::new();
                let mut oldest_message = None;

                let mut requests = 0;
                loop {
                    print_tracking(false);
                    print_wheel(true, requests);

                    let messages = match oldest_message {
                        None => {
                            channel
                                .messages(
                                    &ctx.http, 
                                    |retriever| 
                                        retriever.limit(MESSAGE_BATCH))
                                .await
                                .unwrap_or_else(|err| {
                                    panic!("Couldn't get messages: {}", err);
                                })
                        }
                        Some(message) => {
                            channel
                                .messages(
                                    &ctx.http, 
                                    |retriever| 
                                        retriever
                                            .before(message)
                                            .limit(MESSAGE_BATCH))
                                .await
                                .unwrap_or_else(|err| {
                                    panic!("Couldn't get messages: {}", err);
                                })
                        }
                    };
                    if messages.is_empty() {
                        continue;
                    }

                    let last_batch = messages.len() < MESSAGE_BATCH as usize;
                    oldest_message = messages.last().map(|m| m.id);

                    let attachments = 
                        messages
                        .iter()
                        .filter(|message| {
                            !message.attachments.is_empty()
                        });

                    //let mut images = Vec::new();
                    for message in attachments {
                        for _ in 0..message.attachments.len() {
                            timestamps.push(message.timestamp);
                        }
                    }

                    if last_batch {
                        break;
                    } else {
                        requests = requests + 1;
                    }
                }

                // now compile all the timestamped messages into the daily or weekly statistics
                
                let mut weekly: HashMap<chrono::NaiveDate, u32> = HashMap::new();
                let mut daily: HashMap<chrono::NaiveDate, u32> = HashMap::new();

                for timestamp in timestamps {
                    let day = 
                        NaiveDateTime::from_timestamp_opt(timestamp.unix_timestamp(), 0)
                        .unwrap()
                        .and_local_timezone(Utc)
                        .unwrap()
                        .with_timezone(&Local);

                    let day = day.naive_local();

                    daily.insert(day.date(), daily.get(&day.date()).unwrap_or(&0) + 1);

                    let week = day.date().week(Weekday::Mon).last_day();

                    weekly.insert(week, weekly.get(&week).unwrap_or(&0) + 1);
                }

                // for formatting
                let name = channel
                    .name
                    .replace(|c: char| !c.is_ascii(), "")
                    .replace("-", " ")
                    .trim()
                    .to_string();
                let name = titlecase::titlecase(&name);

                // save the stats in graphs

                saved.insert(
                    name.clone(), 
                    plot(weekly,
                        daily,
                        &name,
                        PathBuf::from("out/"),
                        print_tracking));
            }

            if output_channel.is_none() { continue }
            let output_channel = output_channel.unwrap();

            print!("\x1B[2K\r");
            match Question::new(&format!("Do you want to post these graphs in #{}?", output_channel.name))
                    .confirm() {
                Answer::YES => {
                    for (name, images) in saved.clone() {
                        output_channel
                            .send_files(&ctx.http, &images, |m| {
                                m.content(name)
                            })
                            .await
                            .unwrap_or_else(|err| {
                                panic!("Couldn't send message: {}", err);
                            });
                    }
                },
                _ => {}
            }

        }

        exit(0);
    }
}