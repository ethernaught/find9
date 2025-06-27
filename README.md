find9
====

Find9 is a Rust based DNS server that allows you to easily host your own DNS Name server.

This follows RFC for the most part, except for some small adjustments as the RFC isn't great with all real world scenarios.
For example, RFC allows for more than 1 query per request, however handling for example (AA) in the headers wouldn't make sense
as we don't know what the response is authoritative for. Another example that I have viered from is NS and SOA records should not
contain CName responses as per RFC. However almost all clients accept this and large CDNs like CloudFlare utilize this for faster
lookups.

> [!important]
> This project is not complete

Supported Record Types

| RR Type | Status   | RR Type | Status   |
|---------|----------|---------|----------|
| SOA     | Complete | NS      | Complete |
| A       | Complete | AAAA    | Complete |
| TXT     | Complete | MX      | Testing  |
| AXFR    | Partial  | IXFR    | Partial  |
| OPT     | Partial  | CNAME   | Complete |
| PTR     | Complete | SRV     | Complete |
| CAA     | Todo     | CERT    | Todo     |
| DS      | WIP      | DNSKEY  | Todo     |
| LOC     | Complete | NAPTR   | Todo     |
| SMIMEA  | Todo     | SSHFP   | Todo     |
| SVCB    | Complete | HTTPS   | Complete |
| TLSA    | Todo     | URI     | Complete |
| HINFO   | Complete | ANY     | Complete |
| RRSIG   | Partial  | TSIG    | Todo     |

14 / 28 Complete
4 Partial

This currently supports `.zone` files and will be moved as a library so that you can minipulate the queries to use a DB if you dont want to use a Zone file
Not all Zone methods are working quite yet.

| Zone Type | Status   |
|-----------|----------|
| Master    | Complete |
| Slave     | Partial  |
| Stub      | Todo     |
| Forward   | Todo     |
| Hint      | Complete |

To Do
----

> Ability to generate ECH based off users server private key for HTTPS and SVCB Records

> Implement OPT for EDNS

> Finish IXFR and AXFR (TCP Only)

> Implement DNS not just NS (IE config option for fallback / recursive)

> Calculate serial for SOA records

> Max answers isnt functioning as it should, it only limits to max of specific type

> ECDSA Curve P-256 with SHA-256 code for DS Records
