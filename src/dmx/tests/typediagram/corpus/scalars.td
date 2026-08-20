# Every primitive and semantic scalar, plus every container built-in.
type Scalars {
  flag:      Bool
  count:     Int
  ratio:     Float
  text:      String
  blob:      Bytes
  nothing:   Unit
  at:        DateTime
  id:        Uuid
  amount:    Decimal
  tags:      List<String>
  index:     Map<String, List<Option<Decimal>>>
  maybe:     Option<Int>
  anything:  Any
  deep:      Option<Option<Map<Uuid, List<Any>>>>
}

# A declaration shadows a built-in scalar, which must keep working.
alias Uuid = String
