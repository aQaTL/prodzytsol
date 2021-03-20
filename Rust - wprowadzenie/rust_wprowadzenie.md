## Wprowadzenie do Rusta, dla tych, którzy już trochę programować umieją

Maciej Sołtys

---

### Wersja dla tych, którzy umieją, czyli

- Szybki przegląd składni, typów
- Ficzery
- Różnice (C++ / Java / C# / Go)

---

# Rust


tutaj jakiś wstęp / najważniejsze punkty

---

Live demo

---

## Składnia, podstawowe typy

- zmienne
- mutowalność
- type inherence
- ify
- funkcje
- struktury
- enumy

---

## `for` loop

nie ma

----

## `do while` loop

Nie ma

----

## `while` loop

```rust
let mut a = 0;

while a < 10 {
	a += 1;
}
```

----

## for each loop & Iterator




----

Przykład

---

## Konstruktory / inicjalizacja

Only one way

```rust
struct Foo {
	x: i64,
}

struct Bar(Foo);

enum A {
	One,
	Two,
}

fn foo() {
	let foo = Foo {
		x: 1,
	};
	
	let bar = Bar(foo);
}
```

---

## `enum`

```rust
enum SqrtResult {
	Success(f64),
	Fail(SqrtError),
}

enum SqrtError {
	NegativeNumber,
}

fn sqrt(n: f64) -> SqrtResult {
	if n < 0.0 {
		return SqrtResult::Fail(SqrtError::NegativeNumber);
	}
	
	let sqrt_result = n.sqrt();
	
	Ok(sqrt_result)
}
```

---

## Result

```rust
enum Result<T, E> {
	Ok(T),
	Err(E),
}
```

---

## Null

Nie ma.

----

Nie ma.

----

### `Option`

```rust=
enum Option<T> {
	Some(T),
	None,
}
```

----

Tutaj demo Resulta i Optiona

---

# cargo

- menedżer pakietów / zależności
- system budowania projektu

---

