# Skop - System Knowledge Operations Platform

Skop makes investigating and debugging UNIX systems hurt less. 

![skop](./skop.png)

## Usage
```
git clone https://github.com/zmaril/skop.git
cd skop
cargo run
```

## Philosophy 

Skop is the result of me working ten years as an administrator of various systems and finding myself increasingly dissatisfied with the tools at my disposal. I've learned enough bash and command line to hate it something fierce and then enough on top that all that is left is a dull distaste. I find myself running into the same types of issues time and again and I grow tired of them. 

* When I am debugging, I am tired of losing significant results in the backscroll. A line flies by in the terminal that cracks the issue wide open and there it goes, off to never be seen again, overflowing past the end of the line buffer. 
* Synchronizing and correlating outputs in experiments across time is a pain. Did that line print out before or after I sent the packet from the other host? Who knows! Time to squint at the timestamps and reconstruct the flow in my head.   
* Collaborating on investigative experiments with others is a pain. My investigative scripts are always an afterthought and I resort to verbally sharing anything about the experiments beyond some bad graphs and copy-and-pasted text output. 
* I am no longer in love with knowing stuff about computers. At one point, yes, I did know how to use lsblk and friends, but that knowledge has long been replaced by fond memories of friends, family, and tasty recipes I might like to cook again. 
* The command line has always been a flat wall, with no handles on it beyond the ones you bring yourself, illegible to those who have not spent many hours bashing their hand against learning how to use it. It affords nothing to the user beyond power over the system, a promise that you can change things, for better and, often, for worse. 
* Beyond the known unknowns that a beginner comprehends that they don't know, there are a number of unknown unknowns that beginners cannot fathom. A group of senior engineers surprised others talking with a junior engineer when we offhandedly mentioned that you can often tell what's wrong with a system by logging into it and feeling how the command line responds to you. Certainly if it doesn't connect, you have networking or auth issues, but what does it mean when the cursor blinks slower than expected once you are in? When you run a command, no matter which one it is, and it just hangs? When the input lags behind your keystrokes so much that you notice it? All these things build up after years, little clues that are obvious signs of where to look to those who have been in the game long enough to know what's up. 
* In the cloud, nobody can hear the modems scream, and the CPU's whir. I miss the days when the computer was right next to me and I could hear it huffing and puffing as it tried to climb the mountain I set it on. 

Rather than work harder or change my behavior in any significant or corrective way, I decided to build a tool that has a long shot chance of actually addressing these issues instead, like any senior engineer would. The goal is a tool that does the following: 

* Built around the command line - there's no escaping the command line, eventually every investigation into a thorny problem turns into somebody with a shell open running a series of arcane commands. Can't change that, so embrace it and assume that the user will be using command line tools to investigate the issue. 
* [Infinite canvas](https://jsoncanvas.org/) - the infinite canvas' of late have been nice, for arranging things as you go and making up your own workflow.
* Easy graphing and pivot tables - They're useful! Would be nice if they were built in and I didn't have to figure out excel or matplotlib.
* Prebuilt dashboards and workbooks - Wouldn't it be nice if all the stuff [brendann gregg does](https://www.brendangregg.com) was just a button click away? 
* Record everything - if everything is on the command line, then it's easy enough to 
* Playback anything - if everything is recorded with a local timestamp, then we can travel back in time and see how things developed, focus in on the important parts, slowing down or speeding up as need be to get the exact moment we want to see. 
* Automatically self documenting - with everything recorded, we can send others our skop saves file and they see for themselves what we did and how we did it. 
* Export results - Make it easy to compile our results into something that others can read and understand outside of skop. 
* Hear the system - simulate the whir of the CPU, the little pings of the network, the disk spinning of the file system. skop will allow people to hear what's going on. 
* SSH Bastion for real time collobration - let people log into your session, use the command line outside of skop. 

I think if I had something like this, I would be much happier and so I am burning a few lakes with Claude to see if it make it. 


## Inspiration 

I've been thinking about this sort of thing off and on for a decade now. I tried something like this with [bpfquery](https://github.com/zmaril/bpfquery) but it never felt quite right. Then I saw a video of [Peter Whidden talking about his very cool thing called Mote](https://www.youtube.com/watch?v=Hju0H3NHxVI) and thought maybe I should try something like that. 

## Current Features

Some to none of the above, just an experiment so far. 

## Roadmap & Development & Contributing

Using claude to write a lot of this. Keeping track of things in todo.txt.  File a PR if you like, though I reserve the right to ignore or summarily reject it without reason or comment.

## License

MIT