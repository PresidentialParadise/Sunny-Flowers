# Sunny Flowers
[![Github Workflows](https://img.shields.io/github/workflow/status/Druue/Sunny-Flowers/Docker?logo=github&style=for-the-badge)](https://github.com/Druue/Sunny-Flowers/actions/workflows/docker-publish.yml)
[![Rust 1.55.0+](https://img.shields.io/badge/rust-1.55.0+-93450a.svg?style=for-the-badge&logo=rust)](https://blog.rust-lang.org/2021/09/09/Rust-1.55.0.html)

Sunny Flowers is a Discord bot to play media in voice channels. It uses [Serenity] and [Songbird] to accomplish this.

[Serenity]: https://github.com/serenity-rs/serenity
[Songbird]: https://github.com/serenity-rs/songbird

## Running
You can run Sunny using `cargo run --release`  
When running Sunny locally, she can take in the `DISCORD_TOKEN` via a `.env` file.

## Deployment
For deploying Sunny a `Dockerfile` and [kubernetes](./k8s/deployment.yml) config are provided.  
This works like normal and requires the `DISCORD_TOKEN` present in the environment.

## Roadmap
See the [open issues](https://github.com/Druue/Sunny-Flowers/issues) for a list of proposed features (and known issues).

## Contributing
Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Contact
Sophie - [Ailbe#7190](https://discord.com/users/124008534693117954)
