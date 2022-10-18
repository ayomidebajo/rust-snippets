Make terminal request mic access to be able to read input device

Ideally 
Download sox using `brew` or `apt-get` depending on your os.
Then run `sox -d -d`

Check your security permissons for terminal asking for mic access or just simply click "yes" to the prompt after you run the program above.


After doing the above
run `cargo run DEVICE_NAME` (i.e input device name as it in your mic/speakers settings)