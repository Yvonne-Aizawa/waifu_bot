# RAINY but open source
have seen this [video][yt-video]? i am making her but in rust and worse. 

### what is here
she can talk via telegram

she can send photos on the users request, (using triggerwords)

persistant short term memory

you can ask her for the current time

calendar access (nextcloud)
### what is not here yet.


weather access

long term memory,

### how to run
**note this should work but i am not sure**
1. first make sure rust is installed 
2. create a telegram bot by contacting [bot father][telegram-bot-father]
3. fill in example.ini and copy it to config.ini
4. start a stable diffusion ([AUTOMATIC1111]) server and a text ([oobabooga][oobabooga]) server personally i run it on [runpod.io][runpod] (referral link) i use this template [bloke][bloke] (referral link)

5. update the url in the config.ini
6. start the bot with cargo run in the directory
7. first time might take a while to build the binary

**note that i have only tested it on linux if it does not work on windows open an issue**
### contributing
if you want to fix this code please do. 

Pull request are welcome.

[bloke]: https://runpod.io/gsc?template=f1pf20op0z&ref=yp8enpey
[runpod]: https://runpod.io?ref=yp8enpey
[telegram-bot-father]: https://t.me/BotFather
[AUTOMATIC1111]: https://github.com/AUTOMATIC1111/stable-diffusion-webui
[oobabooga]: https://github.com/oobabooga/text-generation-webui
[rust-install]: https://www.rust-lang.org/tools/install
[yt-video]: https://www.youtube.com/watch?v=OvY4o9zAqrU
