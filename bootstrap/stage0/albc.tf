resource "aws_iam_policy" "policy" {
  name   = "AWSLoadBalancerControllerIAMPolicy"
  policy = file("${path.module}/albc_iam_policy.json")
}
