# DENVER

### A dotenv replacement

## MOTIVATION
Say you work on a web app. You probably have a local development database, one or more dev/staging/prod databases, a few sets of API keys, etc. If you're like me, you use [dotenv](https://pypi.org/project/python-dotenv/) or [dotenv](https://www.npmjs.com/package/dotenv) or [dotenv](https://github.com/bkeepers/dotenv) to handle your local config. If you're also like me, you're tired of manually editing your `.env` file every time you want to switch environments. `denver` is a little tool that
lets you not have to do that.


## USAGE
Put everything you want shared or default among all your environments in your normal `.env` file. Put your env-specific stuff in `.$ENV_NAME.env` files, e.g. `.dev.env`, `.staging.env`, etc. Run your app with `denver -e $ENV_NAME -c $APP`, e.g. `denver -e dev -c venv/bin/python flask run`. `denver` merges your environment variables together, over-writing "older" variables. By default (but see below), `.env` is treated as the "oldest" set of variables, so if you define e.g. `DATABASE_URL` in
`.env` and in `.dev.env` and pass `-e dev`, the value from `.dev.env` will be used.

Env file names are treated case-insentively, so `denver -e DEV` works the same as `-e dev`.

### Advanced
You can merge multiple `.env` files by passing `-e $WHATEVER` multiple times; the "older" rule applies in order, so `-e staging -e dev` would result in anything in `.env` that's also in `.staging.env` getting clobbered, and anything in `.staging.env` that's also in `.dev.env` being clobbered.

You can reverse this by passing `-l`/`--merge-left`.

You can also temporarily set individual values by passing `-s KEY=VALUE` (or `--set KEY=VALUE`). You can pass `-s` multiple times.


### TODO
- [ ] add `--no-clobber` vel sim.
