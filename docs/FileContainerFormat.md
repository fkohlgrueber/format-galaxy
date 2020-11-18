## FileContainerFormat

FormatGalaxy files are usually stored in a common container format. These files should use the ".fg" extension.

A FormatGalaxy file consists of three parts: a common prelude, a format identifier and the payload of the file. 
- `prelude`: The prelude consists of the string "FMTGALv1" encoded in ascii.
- `format_id`: The format identifier is a 64-bit little-endian unsigned integer identifying the inner format.
- `payload`: The payload contains the actual data and is expected to conform to the format specified by `format_id`

### Format

```
<prelude><format_id><payload>
```

### Example

This is how a file using the format with id 5 and containing the ascii text "ABCD12345!" looks like when encoded as a FormatGalaxy container:

```
|-------FMTGALv1-------|------Format id 5------|--Payload (text) ABCD12345!--|
46 4D 54 47 41 4C 76 31 05 00 00 00 00 00 00 00 41 42 43 44 31 32 33 34 35 21
```
