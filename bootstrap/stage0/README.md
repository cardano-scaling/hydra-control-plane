## Manual steps

### Prerequisites

1. Create ALBC IAM policy (this should be created once for the whole account,
   not per cluster).

   ```
   aws iam create-policy \
     --policy-name AWSLoadBalancerControllerIAMPolicy \
     --policy-document file://albc_iam_policy.json
   ```

### Cluster creation steps

1. Create cluster. You must modify the `.metadata.name`, `.metadata.region` and
   `.managedNodeGroups[].availabilityZones` values accordingly.

   ```
   eksctl create cluster -f cluster.yml
   ```
2. Install ALBC via helm.

   ```
   helm install aws-load-balancer-controller eks/aws-load-balancer-controller \
       --set clusterName=YOUR_CLUSTER_NAME \
       --set serviceAccount.create=false \
       --set region=YOUR_REGION_CODE \
       --set vpcId=$(aws eks describe-cluster --name $YOUR_CLUSTER_NAME --region YOUR_REGION_CODE | jq -r '.cluster.resourcesVpcConfig.vpcId') \
       --set serviceAccount.name=aws-load-balancer-controller \
       -n kube-system
   ```
3. Create SSL cert on the corresponding region using AWS Cert Manager (you will
   need the ARN to set up the ingress controller on `stage1`).

   1. Go to Cert manager (in the corresponding region).
   2. Click Request
   3. Choose `Request a public certificate`, then `Next`.
   4. Type the domain. `*.{region}.hydra-doom.sundae.fi`. Add the `sundae-labs:cost-allocation:Service: hydra-doom` tag. Click `Request`.
   5. Click `Create records in Route 53`.
   6. Click `Create Records`.
   7. Wait for certificate to be deemed valid.
