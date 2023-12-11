# Pony
## Yet another UI framework

## Svelte-like Syntax, Rust Idiomatics

We heavily borrow ideas from the Svelte framework.

### JSX Syntax

A declarative syntax that many are already familiar with.


```jsx
<Modal>
    <modal::Title>
        Delete <T mono>system32</T>?
    </model::Title>
    <model::Description>
        This may cause irrecoverable damage to your computer? <Br />
        Do you wish to continue?
    </model::Description>
    <model::Footer>
        <modal::Action secondary>
            <Button>
                <icon::Cross /> {i11n("GENERIC.CANCEL")}
            </Button>
        </model::Action>
        <model::Action primary>
            <Button>
                <icon::TrashCan /> {i11n("SYSTEM32_MODAL.DELETE")}
            </Button>
        </modal::Action>
    </modal::Footer>
</Modal>
```

Our aim is to create a superset of the JSX specification, so if it's valid JSX (and valid Rust) &mdash; it just works.

## Our JSX Extensions

Some common features from Svelte are included too &mdash; with an additional Rust flair!

### The `<script>` block

Nearly all of the imperative code of a component will live in the `<script>` block of a component.

By default, this language is set to Rust (other languages may or may not be supported in the fut).
You can interop with other Rust code

#### Pure functions and methods
* Any function that references a property immeadiately becomes an instance method on `&mut self` (`fn ___(&mut self, ...)`) [^0]:
    ```rust
    // fruit_basket.rs
    #[derive(Default)]
    extern {
        let mut basket : HashMap<String, usize> = Default::default();
    }


    ///
    /// Add a fruit to this basket.
    /// 
    /// This function is public, so we can access anywhere -- usual Rust
    /// access occurs here too.
    /// 
    pub fn add(fruit: impl ToString) {
        *basket.entry(fruit.to_string())
            .or_insert(0) += 1;
    }
    ```
    ```jsx
    // Later on...
    <Card>
        <Title>Fruit bowl with {basket.len()} fruits</Title>
        <!-- List what's in this basket -->
        {#for (fruit, count) in basket.iter()}
            <Icon id={fruit} /> {fruit} count: {count} <Br />
        {/for}
    </Card>
    ```
    And so, if referenced externally:
    ```rust
    // other_file.rs
    use super::fruit_basket::FruitBasket;

    // We've implemented `Default` for this component, so let's use it!
    // Notice this is marked `mut`, but not `extern` -- therefore, this is internal state. 
    let mut basket: FruitBasket = Default::default();

    // Pure function to pick a random fruit from a list.
    fn random_fruit() -> String {
        // todo!("Do the random stuff!")
        "Apple".to_string()
    }
    ```
    ```jsx
    // Later on...
    <Button on:click={|| basket.add(random_fruit())}>Add a fruit</Button>
    <FruitBasket bind:self={basket} />
    ```
* Otherwise, the function is made into an associated function (similar to a `static` function in other languages):
    ```rust
    // queue.rs

    #[derive(Default)]
    extern {
        let mut items: Vec<String> = Default::default();
    }

    ///
    /// Joins a str iterator into a single string.
    /// 
    /// Must have #![feature(intersperse)] to enable the std feature
    ///     (+ be on nightly).
    /// 
    pub fn joined<'a>(iter: impl Iterator<Item = &'a str> + 'a) -> String {
        iter.intersperse(", ")
            .fold(String::new(), |s, a| s + a)
    }
    ```
    ```jsx
    // Later on...
    <T>Queue ({items.len()}): {joined(items.iter())}</T>
    ```
    And, so referenced externally:
    ```rust
    // helper.rs
    
    use super::queue::Queue;

    let lines: Vec<String> = vec![
        "Eat".to_string(), 
        "Code".to_string(), 
        "Sleep".to_string(),
        "Repeat".to_string(),
    ];

    let unfunny_tshirt = Queue::joined(lines.iter());

    println!("Yo, guys! I got a really cool idea for a t-shirt.");
    println!("It'll have the lines: '{unfunny_tshirt}' on it. Isn't that funny?");
    ```
#### Other Items
Nearly everything else behaves the same as it would in normal Rust code:
* Type Aliases
* Sub-modules
* Trait/Struct/Enum definitions
* Static/Const items

All work as Ferris intended.

`impl` blocks are mostly fine, as long as they do not `impl` traits that the transpiler will automatically `impl`.
#### Component Props

* Just like in Svelte, we use the `let` keyword to define names for these properties.
* Default values can be specified by assigning values to these props.
* For now, type inference ***is not*** supported &mdash; specify a type.
* Component props are declared `extern` [^1], and can be declared one at a time, or in a group using an `extern` block. [^1a]

    ```rust
    use my_cool::library::Item;

    // Declaring a single property, with no default:
    extern let inventory: Vec<Item>;

    // Declaring multiple props at once:
    extern {
        // Perhaps the more Rust-like way of dlecaring a default value for a pro:.
        let selected_item: Option<&Item> = Default::default();
        
        // Default health of 10:
        let health: i32 = 10;
    }
    ```
##### Mutability
In normal Rust, the `mut` keyword distinguishes between mutable and immutable variables. This carries over here too: *any prop not declared `mut` cannot be changed after initialization*. In theory, this should help you determine which props are storing actual state.

* This is legal:
    ```rust
    extern let mut score: i32 = 0;
    ```
    ```jsx
    // Later on ...
    <Button on:click={|| score += 1 }>
        Click to increment score: {score}
    </Button>
    ```
* This is not:
    ```rust
    // Missing `mut`
    extern let score: i32 = 0;
    ```
    ```jsx
    // Later on ...
    // ERROR:   Cannot assign twice to immutable variable `score`.
    //          Consider making this binding mutable `mut score` (refers to `extern` block)  
    //                   ↓↓↓↓↓↓↓↓↓↓
    <Button on:click={|| score += 1 }>
        Score {score}
    </Button>
    ```

##### Macros

* All `extern` prop declarations support outer Rustdoc (`///`, not `//!`), so they work mostly as you'd expect:
    ```rust
    /// Player's remaining health points.
    extern let health: i32;

    // Only use these outer docs on an `extern` group to document
    // the props struct itself:

    ///
    /// Player's ability scores.
    ///
    extern {
        /// STR
        let strength: i32;
        /// CON
        let constitution: i32;
        /// INT
        let intelligence: i32;
        /// WIS
        let wisdom: i32;
        /// DEX
        let dexterity: i32;
        /// CHA
        let charisma: i32;
    }
    ```

* When using `struct`-level macros, you must apply them to an `extern` *block*:
    ```rust
    use serde::Deserialize;

    ///
    /// Plugin from the official plugin repository.
    ///
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    extern {
        let id: String;

        ///
        /// They'll never see it comin'...
        ///
        #[serde(default = "https://www.youtube.com/watch?v=dQw4w9WgXcQ")]
        let homepage: String;
    }
    ```
    When using `struct`-level macros, it's advisable to put every prop into in one `extern` block to maintain clarity [^2].


### Logic Blocks

#### `#if` 

You can use `#if` for normal boolean expressions;

```svelte
{#if project.language != "Rust"}
    🦀 says, "Rewrite it in Rust!"
{:else}
    🦀 is proud of you.
{/if}
```

Or, you can use the `if let ... = ...` pattern syntax:

```svelte
{#if let Some(ref chicken) = self.fridge.get("chicken")}
    I have {chicken.weight} kgs of chicken.
{:else}
    No chicken?!
{/if}
```

#### `#for`

`#for` allows you map through an iterator as `.map(...)` does.

```svelte
{#for animal in ark.animals.iter()
    .filter(|a| a.is(Mammal))
}
    <Card>
        <Image
            src={&animal.img} 
            a11y:alt="Picture of {&animal.species}"
        />
        <Title>{&animal.name}</Title>
    </Card>
{/for}
```

`#each` from Svelte has been replaced with `#for` to better fit Rust idiomatics.

#### `#async` [^3]

These behave similarly to Svelte's `#await` blocks, except for the naming.


```rust
{#async Cookbook::fetch_recipes()}
    Loading cool recipes... 
    <!-- TODO: Make cool animation. -->
{:await recipes}
    {#for recipe in recipes.into_iter()
        .map(Recipe::deserialize)
        .filter_map(Result::ok)
    }
        <dyn Component self={recipe}/>
    {/for}
{/async}
```

Notice the two distinct sections: before `{:await ...}`, and after.
The markup declared before `{:await ...}` will be displayed until the future is `Ready(...)`, then the markup after is displayed.

* The `{:await ...}` divider can be omitted if you do not care about the output of the future, when finished.
* You can use the shorthand `{#async let output = fut.await}` to only display the matkup in the `async` block when `fut` has finished (its output will be put into variable `output`) in this example:
```rust
<!-- Without {:await ...} divider. --> 
{#async stocks.finished()}
    <Button on:click={|| panic!()}>Click me</Button>
    <T>Panic-buy Ferris plushies whilst stocks last!</T>
{/async}

<!-- Using let-await shorthand. --> 
{#async let order = user.latest_order().await }
    Your latest order for {order.qty} X {order.item} has been processed.
{/async}
```

#### `#match`

These are completely new. These allow you to utilize Rust's
powerfull pattern matching in markup.

We introduce the `case` keyword to form the divider.

Note, that a `{:case ...}` must immeadiately follow the `{#match ...}` opening -- nothing else can go in between (except comments).

```rust
{#match user.role()}
    {:case Role::Root}
        <T>With great power comes great responsibility.</T>
    {:case Role::Sudoer}
        <T>You are now (effectively) running as root.</T>
    {:case _}
        <T>{user.username} is not in the sudoers file.</T>
        <T>This incident will be reported.</T>
{/match}
```

Again, we can use complicated pattern matching as usual:
```rust
<!-- Only show age-appropriate action films. -->
{#match film}
    {:case Film(title, age_rating @ ..=user.age(), Genre::Action)}
        <Poster>
            <Title>{title}</Title>
            <AgeRating>{age_rating}</AgeRating>
            <GenreIcon genre={Genre::Action} />
        </Poster>
{/match}
```

#### `{#key}` [^4]

This behaves exactly like Svelte's `{#key ...}` block does: this entire block will be destroyed then re-created when the expression changes:

```svelte
{#key month}
    <!-- Green Day would be proud! -->
    Wake me up when {month} ends.
{/key}
```

### Tags

These are special directives that allow for extra functionality whilst in markup.
They are all of syntax `{@ ...}` and are ideally one-liners.

#### Debugging
You can use the `print!`, `println!`, `panic!` macros as usual by simply putting them inside the `{@}`:

```rust
<Bookshelf>
    {#for book in shelf.books()}
        <Book>
            {@println!("ISBN-10: {}", book.isbn_10())}
        </Book>
    {/for}
</Bookshelf>
```

`{@debug expr}` is reserved for a potential custom breakpoint-based debugger (similar to `debugger` in JavaScript).

#### `{@let ...}`
This behaves exactly as you'd expect:
```rust
<Song>
    <Artist>Robbie Williams</Artist>
    <Lyrics>
        {@let words = "Let me entertain you!"}
        {words}
    </Lyrics>
</Song>
```

### Mustaches

You've seen them all over the place right now.

Mustaches `{expr}` allow for applicable values to be interpolated into strings.

When interpolating, we use the `std::fmt::Display` trait by default to perform the convertion (as expected).

However, we support most the format! macro placeholder formatting syntax:
```rust
{@let price : f64 = 420.6969}
<T>That'll be ${price:.3}</T>
```

Not supported:
* `$` formatting variables
* integer inddex parameters, such as `format!("{0} {1}, or not {0} {1}", "to", "be")`


[^1]  Should `extern` be the keyword to declare this, or should there be a `props! { ... }` macro?
[^1a] Should mixing singleton `extern let ... = ...;` and `extern` block prop declarations be allowed?
[^2]  Should this be a compiler warning or error?
[^3]  Naming: is `async` the best word to use here?
[^4]  Naming: is `key` the best keyword to use, over something like `watch`?
