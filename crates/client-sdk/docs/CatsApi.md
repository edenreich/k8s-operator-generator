# \CatsApi

All URIs are relative to *http://localhost:8080*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_cat**](CatsApi.md#create_cat) | **POST** /cats | Create a new cat
[**delete_cat_by_id**](CatsApi.md#delete_cat_by_id) | **DELETE** /cats/{id} | Delete a cat by ID
[**get_cat_by_id**](CatsApi.md#get_cat_by_id) | **GET** /cats/{id} | Get a cat by ID
[**get_cats**](CatsApi.md#get_cats) | **GET** /cats | Get all cats
[**update_cat_by_id**](CatsApi.md#update_cat_by_id) | **PUT** /cats/{id} | Update a cat by ID



## create_cat

> models::Cat create_cat(cat)
Create a new cat

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**cat** | [**Cat**](Cat.md) |  | [required] |

### Return type

[**models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_cat_by_id

> delete_cat_by_id(id)
Delete a cat by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_cat_by_id

> models::Cat get_cat_by_id(id)
Get a cat by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |

### Return type

[**models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_cats

> Vec<models::Cat> get_cats()
Get all cats

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Cat>**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_cat_by_id

> models::Cat update_cat_by_id(id, cat)
Update a cat by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** |  | [required] |
**cat** | [**Cat**](Cat.md) |  | [required] |

### Return type

[**models::Cat**](Cat.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

