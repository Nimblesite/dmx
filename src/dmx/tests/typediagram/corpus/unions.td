union Shape {
  Circle    { radius: Float }
  Rectangle { width: Float, height: Float }
  Triangle  { a: Float, b: Float, c: Float }
  Point
}

union ErrorCode {
  ParseError = -32700
  InvalidRequest = -32600
  MethodNotFound = -32601
  Ok = 0
  Grouped = 1_000
}

union Option<T> {
  Some { value: T }
  None
}

union Result<T, E> {
  Ok  { value: T }
  Err { error: E }
}

union RequestId {
  Number(Int)
  String(String)
  Triple(Int, String, List<Bool>)
}

untagged union Loose {
  Left  { value: Int }
  Right { value: String }
}
