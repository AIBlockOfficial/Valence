<div id="top"></div>

<!-- PROJECT LOGO -->
<br />

<div align="center">
    <div style="height: 50px; width: 100%"></div>

  <a>
    <img src="https://github.com/ABlockOfficial/Weaver/blob/main/assets/hero.svg" alt="Logo" width="320px">
  </a>

  <div style="height: 50px; width: 100%"></div>

  <h3>Weaver</h3>

  <!-- <div>
  <img src="https://img.shields.io/github/actions/workflow/status/Zenotta/Intercom/codeql-analysis.yml?branch=main" alt="Pipeline Status" />
    <img src="https://img.shields.io/github/package-json/v/Zenotta/Intercom" />
  </div> -->

  <p align="center">
    The A-Block L2 node for data exchange between peers. Complete with E2E encryption.
    <br />
    <br />
    <a href="https://a-block.io"><strong>Official documentation »</strong></a>
    <br />
    <br />
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#running-the-server">Running The Server</a></li>
        </ul>
    </li>
    <li>
      <a href="#how-it-works">How it Works</a>
      <ul>
        <li><a href="#data-exchange">Data Exchange</a></li>
        <li><a href="#further-work">Further Work</a></li>
        </ul>
    </li>
  </ol>
</details>

<!-- GETTING STARTED -->

## Getting Started

### 📚 Prerequisites

In order to run this server as a community provider, or simply to use it yourself, you'll need to have <a href="https://www.docker.com/products/docker-desktop/">Docker</a> installed (minimum tested v20.10.12) and be comfortable working with the command line. 

If you'd like to develop on this repo, you'll have the following additional requirements:

- **Rust** (tested on 1.68.0 nightly)

..

<p align="left">(<a href="#top">back to top</a>)</p>

..

### 🔧 Installation

With Docker installed and running, you can clone this repo and get everything installed with the following:

```sh
# SSH clone
git clone git@gitlab.com:ABlockOfficial/Weaver.git

# Navigate to the repo
cd Weaver

# Build Docker image
docker build -t weaver .
```

<p align="left">(<a href="#top">back to top</a>)</p>

..

### 🏎️ Running

To use the server as is, you can simply run the following in the root folder of the repo:

```sh
docker-compose up -d
```

Docker will orchestrate the node itself, the Redis instance, and the MongoDB long-term storage, after which you can make 
calls to your server at port **3030**. Data saved to the Redis and MongoDB instances is kept within a Docker volume.

To run the server in a development environment, run the following command:

```sh
cargo build --release

cargo run --release
```

<p align="left">(<a href="#top">back to top</a>)</p>

..

## How it Works

The server functions on a very basic set of rules. Clients exchange data between each other through the use of public key addresses. If Alice wants to exchange data with Bob, she will need to supply the Weaver node with Bob's address, as well as her own address, public key, and signature in the call headers. The next time Bob fetches data from the server using his public key address, he would find that Alice has exchanged data to him.

<p align="left">(<a href="#top">back to top</a>)</p>

..

### Available Routes

#### `set_data`
Sets data in the Redis instance and marks it for pending retrieval in the server. To send data to Bob, we could use the following headers in the `set_data` call:

```json
{
    "address": "76e...dd6",     // Bob's public key address
    "public_key": "a4c...e45",   // Alice's public key
    "signature": "b9f...506"     // Alice's signature of Bob's address, using his public key
}
```

The body of the `set_data` call would contain the data being exchanged:

```json
{
    "data": "hello Bob"
}
```

The headers that Alice sends in her call will be validated by the Weaver, after which they'll be stored at Bob's address for his later retrieval using the `get_data` call.

..

##### `get_data`
Gets pending data from the server for a given address. To retrieve data for Bob, he only has to supply his credentials in the call header:

```json
[
    {
        "address": "76e...dd6",     // Bob's public key address
        "signature": "b9f...506",   // Bob's signature of the public key
        "public_key": "a4c...e45"   // Bob's public key corresponding to his address
    }
]
```

Again, the Weaver will validate the signature before returning the data to Bob.

**For best practice, it's recommended that Alice and Bob encrypt their data using their private keys, before exchanging it with each other.** This ensures that the data exchange is E2E encrypted, and that the Weaver maintains no knowledge of the data's content.

<p align="left">(<a href="#top">back to top</a>)</p>

..

### Further Work

- [ ] Add a rate limiting mechanism
- [ ] Set Redis keys to expire (handle cache lifetimes)
- [ ] Handle data storage over time
- [ ] Add tests

<p align="left">(<a href="#top">back to top</a>)</p>

..
