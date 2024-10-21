# Meal Planner App using egui

A pet project that I am using on a weekly basis. It has helped me save money, lose body fat and avoid taking food supplements. Aside from all these wonderful benefits, I learned a lot about Rust, WebAssembly and egui to the point where 
I am seriously considering having all my future pet projects in Rust and a GUI framework that targets WebAssembly (cxx-qt or egui).

See the [demo](https://dejang.github.io/meal-planner-egui/)

## Building from source
```
cargo build && cargo run
// wasm
trunk serve
```

## Note to OSX users
Currently some versions of MacOSX have issues with the panel sizes on first run. The way this manifests is the Central Panel which holds the Recipe Browser and currently viewed recipe occupies the entire screen. 
Because the panels are resizable this can be fixed by resizing this panel horizontally to allow for the Meal Planner to come into the viewport. Just move your mouse to the bottom of the screen and pull up the panel resize bar. 
This is a one time fix, the settings will be remembered on your browser or MacOS desktop unless you purposely remove them.

## Motivation
Rust is cool! Meal planning is cool! Meal planning is healthy! But meal planning takes a lot of time if you are trying to have your diet support your athletic performance while also hitting your target macros and micros on a daily basis. 
Most, if not all, applications currently available are for tablets and phones. And they have a similar layout which often times does not help getting a birds-eye view of your diet. 
Besides that, introducing a recipe from a book, or website, into these apps using the phone keyboard is a nightmare. I've tried it and I gave up, my fingers hurt. I want
to quickly be able to introduce ingredients, change them, play with proportions and serving sizes until I get the results I want for a meal and I can fit it in my plan. 

So this is what this app tries to address: it provides a calendar-like view for the upcoming 6 days on the bottom half of the screen. Use the top half as a browser to quickly find what recipes will fit your restrictions. Using a drag-n-drop
experience you can drag a recipe into the column (day) you want. At the bottom of each day you will see a food-label like view with important things you want to keep an eye out for, such as calories, protein, fats, cholesterol, sodium etc. 


## Always free! Because we should all be healthy!
I don't want to have to subscribe to an app. I find subscription based apps extremely annoying. I get the reasoning of the developers but...seriously?! 
This is why this app will never cost money. Take it, use it as you wish! Be healthy! Be fit! Enjoy delicious meals while making your friend jealous with your silhouette.

## Known issues
Some pictures will not load in the browser due to CORS restrictions. It is what it is, nothing can be done about it with a free hosting service like Github pages.  
This problem does not manifest if you run the app on your Desktop since this is a native desktop application after all, it is not using a WebView (if you figure out how to embed a WebView in egui let me know!).


