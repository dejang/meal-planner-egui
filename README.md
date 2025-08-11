# Meal Planner App using egui

A pet project that I am using on a weekly basis. It has helped me save money, lose body fat and avoid taking food supplements. Aside from all these wonderful benefits, I learned a lot about Rust, WebAssembly and egui to the point where 
I am seriously considering having all my future pet projects in Rust and a GUI framework that targets WebAssembly (cxx-qt or egui).

See the [demo](https://dejang.github.io/meal-planner-egui/)

<img width="1800" height="1031" alt="Image" src="https://github.com/user-attachments/assets/35ec816d-a6df-46a9-a52c-243dccd6fa07" />

## Building from source
```
cargo build && cargo run
// wasm
trunk serve
```

## Features

- copy/paste a list of ingredients in the ingredients box and get immediate nutrients analysis (macros & micros) per serving
- easily save recipes found on the internet in your "recipe book" and attach a descriptive picture
- plan your meals for 6 days using a calendar like view with breakdown of your macros & micros for each day 
- get a shopping list with all the things you need to buy based on your current week plan

## Motivation
Rust is cool! Meal planning is cool! Meal planning is healthy! But meal planning takes a lot of time if you are trying to have your diet support your athletic performance while also hitting your target macros and micros on a daily basis. 
Most, if not all, applications currently available are for tablets and phones. And they have a similar layout which often times does not help getting a birds-eye view of your diet. 
Besides that, introducing a recipe from a book, or website, into these apps using the phone keyboard is extremely time consuming. Ideally, a meal planning application should allow me to copy/paste the list of ingredients, the instructions,
save them and then begin planning. I should also be able to get a shopping list so I don't lose too much time shopping for groceries.

So this is what this app tries to address: it provides a calendar-like view for the upcoming 6 days on the bottom half of the screen. Use the top half as a browser to quickly find what recipes will fit your restrictions. Using a drag-n-drop
experience you can drag a recipe into the column (day) you want. At the bottom of each day you will see a food-label like view with important things you want to keep an eye out for, such as calories, protein, fats, cholesterol, sodium etc. 


## Always free! Because we should all be healthy!
This app will never cost money. Use it for yourself, explore delicious recipes and maintain your fitness goals. As someone who has already tried this for more than 6 months I can tell you this: if it worked for me, it will work for you too!

## Known issues
This APP uses minimal storage. There's a default recipe book (the ones I actually cook for myself) but you can easily modify them or even remove them, if you wish. The changes will be stored in your local storage (WASM) or on your machine (native app).
Because of not having an actual backend layer, when running the app in the Browser you will see that some images are not loading. This is because of CORS restrictions from those domains. There is nothing that can be done about it. If you are annoyed
by this, download the app, compile it using Rust and then run it.

## Contributions
Help me make it better! I am looking forward to see PRs from like-minded people who have the skills to make changes to this app so it can look/feel better.

