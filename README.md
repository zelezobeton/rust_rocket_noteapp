# Rust note webapp build with Rocket, SQLx and Yew 
## Commands
### In frontend folder: 
```
trunk serve
```
### In backend folder: 
__First time__
```
sqlx database create
sqlx migrate run
```
__Run server__
```
cargo run
```

Steps for SQLite DB creation:
* Install sqlx-cli, create .env file with db url, create db and run first migration
* https://github.com/launchbadge/sqlx/tree/master/sqlx-cli
* https://github.com/launchbadge/sqlx/tree/master/examples/sqlite/todos