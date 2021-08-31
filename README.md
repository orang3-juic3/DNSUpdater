##DNS Updater for Cloudflare
***
Simple program that updates DNS records to an ip, or your current ip if none specified.
It's also my first project in rust.

How to use:    
All information should be supplied as args.    
Arg 1: Zone name    
Arg 2: Cloudflare account email    
Arg 3: Cloudflare token    
Arg 4: The record type   
The last arg should be newip=ip_here, 
or it should not be present if switching to your current ip (good for dynamic ips) 