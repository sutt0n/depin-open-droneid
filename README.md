# Description

This is basically a "miner" that scans for Remote ID packets being broadcasted over both Bluetooth LTE Announcements and WiFi Beacons. Those are commented out for now [1], but for the most part, they work (see the unit tests).

I won't be publishing the spec for Remote ID, as it's a document you must purchase; however, you can reverse engineer the spec from the various parsers. 😊

[1] They're commented out because this codebase was part of a project where the focus shifted from scanning drones to a memecoin and I did not have time to allocate as my paternity leave was ending. 

# Why Publish

I wanted to showcase my work. I put a lot of time into this, and I learned a lot and quite frankly, this was a very fun project.

The project ended up going nowhere, as communication and interest fizzled out. 

# What does this do?

If you have a Bluetooth module, it scans for announcement packets over those channels. If you have a WiFI module capable of monitor mode, it puts it into monitor mode and modulates the adapter over a variety of channels that Remote ID is broadcasted over. Keep in mind, there are some dependencies required in order to put the WiFi module into monitor mode. This was used on Ubuntu desktop with a WiFi Module 802.11 b/g/n - emphasis on the "n" frequency band here. This was E2E tested using a DroneTag module (or a drone capable of Remote ID; most DJI drones come with Remote ID so long as they're over the 200g requirement - anything under, and that requirement is nil).

## Does it "Track"?

Yes. Once a packet payload is found, it keeps the WiFi module on that specific channel. Now, you'd technically need _many_ modules for this to work efficiently; however, this miner works with just one. It can be easily modified to cater to multiple WiFi modules. 
