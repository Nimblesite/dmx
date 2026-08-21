typeDiagram

# Every record shape the language reference names.
type User {
  id:      Uuid
  name:    String
  email:   Option<Email>
  roles:   List<Role>
  address: Address
}

type Pair<A, B> {
  first:  A
  second: B
}

type Box<T> {
  value: T
}

type Empty {
}

type Separators { a: Int, b: Int
  c: Int,
}

type Email { text: String }
type Role { name: String }
type Address { line: String }
